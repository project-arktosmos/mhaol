use crate::error::{Error, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Default segment duration in seconds.
pub const DEFAULT_SEGMENT_DURATION: u32 = 6;
/// Default playlist max length (number of segments kept). 0 means keep all
/// segments (VOD-style); the player can seek anywhere inside the file.
pub const DEFAULT_PLAYLIST_LENGTH: u32 = 0;

/// Build a GStreamer element with the given factory and optional name.
fn make_element(factory: &str, name: Option<&str>) -> Result<gst::Element> {
    let mut b = gst::ElementFactory::make(factory);
    if let Some(n) = name {
        b = b.name(n);
    }
    b.build().map_err(|_| Error::ElementNotFound(factory.into()))
}

/// Configuration for an HLS transcoding pipeline.
#[derive(Debug, Clone)]
pub struct HlsPipelineConfig {
    /// Path to the source media file on disk.
    pub source_path: PathBuf,
    /// Directory where the playlist and segments will be written.
    /// Created if it doesn't exist.
    pub output_dir: PathBuf,
    /// Filename of the m3u8 inside `output_dir`. Defaults to `playlist.m3u8`.
    pub playlist_name: String,
    /// Segment filename pattern. `%05d` is replaced with a zero-padded
    /// segment index. Defaults to `segment%05d.ts`.
    pub segment_pattern: String,
    /// Target segment duration in seconds.
    pub segment_duration: u32,
    /// How many segments the playlist keeps. `0` keeps all segments (VOD).
    pub playlist_length: u32,
}

impl HlsPipelineConfig {
    pub fn new(source_path: impl Into<PathBuf>, output_dir: impl Into<PathBuf>) -> Self {
        Self {
            source_path: source_path.into(),
            output_dir: output_dir.into(),
            playlist_name: "playlist.m3u8".into(),
            segment_pattern: "segment%05d.ts".into(),
            segment_duration: DEFAULT_SEGMENT_DURATION,
            playlist_length: DEFAULT_PLAYLIST_LENGTH,
        }
    }
}

/// A built HLS streaming pipeline. Drop to release resources.
pub struct HlsPipeline {
    pipeline: gst::Pipeline,
    output_dir: PathBuf,
    playlist_name: String,
}

impl HlsPipeline {
    pub fn pipeline(&self) -> &gst::Pipeline {
        &self.pipeline
    }

    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    pub fn playlist_name(&self) -> &str {
        &self.playlist_name
    }

    pub fn playlist_path(&self) -> PathBuf {
        self.output_dir.join(&self.playlist_name)
    }

    pub fn play(&self) -> Result<()> {
        self.pipeline
            .set_state(gst::State::Playing)
            .map_err(|e| Error::StateChange(format!("Failed to play: {e}")))?;
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        self.pipeline
            .set_state(gst::State::Null)
            .map_err(|e| Error::StateChange(format!("Failed to stop: {e}")))?;
        Ok(())
    }
}

impl Drop for HlsPipeline {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}

/// Build the HLS transcoding pipeline:
///
/// ```text
/// filesrc -> decodebin -+-> videoconvert -> x264enc ----+-> mpegtsmux -> hlssink2
///                       \                                /
///                        +-> audioconvert -> aacenc ----+
/// ```
///
/// The `decodebin` dynamically wires its source pads to the video/audio
/// branches as it identifies streams. Both branches feed a single
/// `mpegtsmux` whose output goes through `hlssink2`, which writes
/// `playlist.m3u8` plus segment files into `config.output_dir`.
pub fn build_hls_pipeline(config: HlsPipelineConfig) -> Result<HlsPipeline> {
    if !config.source_path.exists() {
        return Err(Error::SourceNotFound(
            config.source_path.display().to_string(),
        ));
    }
    std::fs::create_dir_all(&config.output_dir)?;

    let pipeline = gst::Pipeline::default();

    let filesrc = make_element("filesrc", Some("filesrc"))?;
    filesrc.set_property(
        "location",
        config
            .source_path
            .to_str()
            .ok_or_else(|| Error::PipelineConstruction("non-utf8 source path".into()))?,
    );
    let decodebin = make_element("decodebin", Some("decodebin"))?;
    let mux = make_element("mpegtsmux", Some("muxer"))?;
    let hlssink = make_element("hlssink2", Some("hlssink"))?;

    let playlist_path = config.output_dir.join(&config.playlist_name);
    let segment_path = config.output_dir.join(&config.segment_pattern);

    hlssink.set_property(
        "playlist-location",
        playlist_path
            .to_str()
            .ok_or_else(|| Error::PipelineConstruction("non-utf8 playlist path".into()))?,
    );
    hlssink.set_property(
        "location",
        segment_path
            .to_str()
            .ok_or_else(|| Error::PipelineConstruction("non-utf8 segment path".into()))?,
    );
    hlssink.set_property("target-duration", config.segment_duration);
    hlssink.set_property("playlist-length", config.playlist_length);
    // Keep every segment on disk for VOD-style playback. Without this
    // hlssink2 deletes old segments once the playlist rotates.
    hlssink.set_property("max-files", 0u32);
    // Tell hlssink2 to author a VOD-style playlist with `#EXT-X-PLAYLIST-TYPE:VOD`
    // and an `#EXT-X-ENDLIST` tag once EOS arrives. send-keyframe-requests is on
    // by default and forces keyframes on segment boundaries.

    pipeline
        .add_many([&filesrc, &decodebin, &mux, &hlssink])
        .map_err(|e| Error::PipelineConstruction(e.to_string()))?;
    filesrc
        .link(&decodebin)
        .map_err(|_| Error::ElementLinkFailed {
            source_element: "filesrc".into(),
            sink_element: "decodebin".into(),
        })?;

    // hlssink2 exposes "video" and "audio" request pads. We connect the
    // muxer's `src` pad to hlssink2's `video` pad below if a video branch
    // is added. mpegtsmux already handles A/V mux so we only need to wire
    // its src to hlssink2's video sink.
    let mux_src = mux
        .static_pad("src")
        .ok_or_else(|| Error::PipelineConstruction("mpegtsmux has no src pad".into()))?;
    let hls_video_sink = hlssink
        .request_pad_simple("video")
        .ok_or_else(|| Error::PipelineConstruction("hlssink2 video pad request failed".into()))?;
    mux_src
        .link(&hls_video_sink)
        .map_err(|_| Error::ElementLinkFailed {
            source_element: "mpegtsmux".into(),
            sink_element: "hlssink2".into(),
        })?;

    let pipeline_weak = pipeline.downgrade();
    let mux_weak = mux.downgrade();
    let video_linked = Arc::new(AtomicBool::new(false));
    let audio_linked = Arc::new(AtomicBool::new(false));

    decodebin.connect_pad_added(move |_decodebin, src_pad| {
        let Some(pipeline) = pipeline_weak.upgrade() else {
            return;
        };
        let Some(mux) = mux_weak.upgrade() else {
            return;
        };

        let caps = src_pad
            .current_caps()
            .unwrap_or_else(|| src_pad.query_caps(None));
        let Some(structure) = caps.structure(0) else {
            return;
        };
        let media_type = structure.name().as_str();

        if media_type.starts_with("video/") {
            if video_linked.swap(true, Ordering::SeqCst) {
                return;
            }
            if let Err(e) = link_video_branch(&pipeline, src_pad, &mux) {
                tracing::error!("[ipfs-stream] failed to link video branch: {e}");
                video_linked.store(false, Ordering::SeqCst);
            }
        } else if media_type.starts_with("audio/") {
            if audio_linked.swap(true, Ordering::SeqCst) {
                return;
            }
            if let Err(e) = link_audio_branch(&pipeline, src_pad, &mux) {
                tracing::error!("[ipfs-stream] failed to link audio branch: {e}");
                audio_linked.store(false, Ordering::SeqCst);
            }
        }
    });

    Ok(HlsPipeline {
        pipeline,
        output_dir: config.output_dir,
        playlist_name: config.playlist_name,
    })
}

fn link_video_branch(
    pipeline: &gst::Pipeline,
    src_pad: &gst::Pad,
    mux: &gst::Element,
) -> Result<()> {
    let queue = make_element("queue", Some("video_queue"))?;
    let convert = make_element("videoconvert", Some("video_convert"))?;
    let encoder = make_element("x264enc", Some("video_encoder"))?;
    let h264parse = make_element("h264parse", Some("video_parser"))?;

    encoder.set_property_from_str("tune", "zerolatency");
    encoder.set_property_from_str("speed-preset", "veryfast");
    encoder.set_property("bitrate", 2500u32);
    // Force keyframes on segment boundaries so hlssink2 can cut cleanly.
    encoder.set_property("key-int-max", 60u32);

    pipeline
        .add_many([&queue, &convert, &encoder, &h264parse])
        .map_err(|e| Error::PipelineConstruction(e.to_string()))?;
    gst::Element::link_many([&queue, &convert, &encoder, &h264parse]).map_err(|_| {
        Error::ElementLinkFailed {
            source_element: "video chain".into(),
            sink_element: "video chain".into(),
        }
    })?;
    h264parse.link(mux).map_err(|_| Error::ElementLinkFailed {
        source_element: "h264parse".into(),
        sink_element: "mpegtsmux".into(),
    })?;

    for el in [&queue, &convert, &encoder, &h264parse] {
        el.sync_state_with_parent()
            .map_err(|e| Error::StateChange(e.to_string()))?;
    }

    let queue_sink = queue
        .static_pad("sink")
        .ok_or_else(|| Error::PipelineConstruction("video queue has no sink pad".into()))?;
    src_pad
        .link(&queue_sink)
        .map_err(|_| Error::ElementLinkFailed {
            source_element: "decodebin".into(),
            sink_element: "video_queue".into(),
        })?;
    Ok(())
}

fn link_audio_branch(
    pipeline: &gst::Pipeline,
    src_pad: &gst::Pad,
    mux: &gst::Element,
) -> Result<()> {
    let queue = make_element("queue", Some("audio_queue"))?;
    let convert = make_element("audioconvert", Some("audio_convert"))?;
    let resample = make_element("audioresample", Some("audio_resample"))?;
    let encoder = pick_aac_encoder()?;
    let aacparse = make_element("aacparse", Some("audio_parser"))?;

    pipeline
        .add_many([&queue, &convert, &resample, &encoder, &aacparse])
        .map_err(|e| Error::PipelineConstruction(e.to_string()))?;
    gst::Element::link_many([&queue, &convert, &resample, &encoder, &aacparse]).map_err(|_| {
        Error::ElementLinkFailed {
            source_element: "audio chain".into(),
            sink_element: "audio chain".into(),
        }
    })?;
    aacparse.link(mux).map_err(|_| Error::ElementLinkFailed {
        source_element: "aacparse".into(),
        sink_element: "mpegtsmux".into(),
    })?;

    for el in [&queue, &convert, &resample, &encoder, &aacparse] {
        el.sync_state_with_parent()
            .map_err(|e| Error::StateChange(e.to_string()))?;
    }

    let queue_sink = queue
        .static_pad("sink")
        .ok_or_else(|| Error::PipelineConstruction("audio queue has no sink pad".into()))?;
    src_pad
        .link(&queue_sink)
        .map_err(|_| Error::ElementLinkFailed {
            source_element: "decodebin".into(),
            sink_element: "audio_queue".into(),
        })?;
    Ok(())
}

/// Pick the first available AAC encoder. GStreamer ships several depending on
/// the platform/plugin set: `voaacenc` (gst-plugins-bad), `avenc_aac` (gst-libav),
/// `faac` (gst-plugins-bad with FAAC). Try them in order of quality.
fn pick_aac_encoder() -> Result<gst::Element> {
    for factory in &["avenc_aac", "voaacenc", "faac"] {
        if let Ok(el) = make_element(factory, Some("audio_encoder")) {
            return Ok(el);
        }
    }
    Err(Error::ElementNotFound(
        "no AAC encoder (avenc_aac, voaacenc, or faac) available".into(),
    ))
}
