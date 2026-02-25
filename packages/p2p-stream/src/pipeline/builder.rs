use crate::error::{Error, Result};
use crate::media::{AudioCodec, CodecConfig, MediaSource, VideoCodec};
use crate::pipeline::bus::{spawn_bus_monitor, BusEvent};
use crate::pipeline::elements::make_element;
use gstreamer as gst;
use gstreamer::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;

/// STUN/TURN server configuration for ICE.
#[derive(Debug, Clone)]
pub struct IceServerConfig {
    pub stun_server: Option<String>,
    pub turn_servers: Vec<String>,
}

impl Default for IceServerConfig {
    fn default() -> Self {
        Self {
            stun_server: Some("stun://stun.l.google.com:19302".into()),
            turn_servers: Vec::new(),
        }
    }
}

/// A built GStreamer pipeline ready for WebRTC streaming.
pub struct StreamPipeline {
    pub(crate) pipeline: gst::Pipeline,
    pub(crate) webrtcbin: gst::Element,
    pub(crate) bus_events: mpsc::UnboundedReceiver<BusEvent>,
}

impl StreamPipeline {
    pub fn pipeline(&self) -> &gst::Pipeline {
        &self.pipeline
    }

    pub fn webrtcbin(&self) -> &gst::Element {
        &self.webrtcbin
    }

    pub fn play(&self) -> Result<()> {
        self.pipeline
            .set_state(gst::State::Playing)
            .map_err(|e| Error::StateChange(format!("Failed to play: {e}")))?;
        Ok(())
    }

    pub fn pause(&self) -> Result<()> {
        self.pipeline
            .set_state(gst::State::Paused)
            .map_err(|e| Error::StateChange(format!("Failed to pause: {e}")))?;
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        self.pipeline
            .set_state(gst::State::Null)
            .map_err(|e| Error::StateChange(format!("Failed to stop: {e}")))?;
        Ok(())
    }

    pub fn seek(&self, position_secs: f64) -> Result<()> {
        let position =
            gst::ClockTime::from_nseconds((position_secs * 1_000_000_000.0) as u64);
        self.pipeline
            .seek_simple(
                gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                position,
            )
            .map_err(|e| Error::StateChange(format!("Seek failed: {e}")))?;
        Ok(())
    }

    pub fn query_position_secs(&self) -> Option<f64> {
        // webrtcbin doesn't answer position queries (it's a real-time RTP element),
        // so query the processing queue elements which propagate upstream to filesrc.
        for name in &["video_queue", "audio_queue"] {
            if let Some(element) = self.pipeline.by_name(name) {
                if let Some(pos) = element.query_position::<gst::ClockTime>() {
                    return Some(pos.nseconds() as f64 / 1_000_000_000.0);
                }
            }
        }
        self.pipeline
            .query_position::<gst::ClockTime>()
            .map(|t| t.nseconds() as f64 / 1_000_000_000.0)
    }

    pub fn query_duration_secs(&self) -> Option<f64> {
        for name in &["video_queue", "audio_queue"] {
            if let Some(element) = self.pipeline.by_name(name) {
                if let Some(dur) = element.query_duration::<gst::ClockTime>() {
                    return Some(dur.nseconds() as f64 / 1_000_000_000.0);
                }
            }
        }
        self.pipeline
            .query_duration::<gst::ClockTime>()
            .map(|t| t.nseconds() as f64 / 1_000_000_000.0)
    }

    pub async fn next_bus_event(&mut self) -> Option<BusEvent> {
        self.bus_events.recv().await
    }
}

impl Drop for StreamPipeline {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}

/// Builder for constructing a WebRTC streaming pipeline.
///
/// Pipeline topology:
/// ```text
/// [MediaSource] → decodebin ─┬─ videoconvert → encoder → rtppay ─┐
///                             └─ audioconvert → resample → opusenc → rtpopuspay ─┤
///                                                                                 └─ webrtcbin → DTLS/ICE → network
/// ```
pub struct PipelineBuilder {
    codec_config: CodecConfig,
    ice_config: IceServerConfig,
    pipeline_name: Option<String>,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        Self {
            codec_config: CodecConfig::default(),
            ice_config: IceServerConfig::default(),
            pipeline_name: None,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.pipeline_name = Some(name.into());
        self
    }

    pub fn video_codec(mut self, codec: VideoCodec) -> Self {
        self.codec_config.video = Some(codec);
        self
    }

    pub fn audio_codec(mut self, codec: AudioCodec) -> Self {
        self.codec_config.audio = Some(codec);
        self
    }

    pub fn no_video(mut self) -> Self {
        self.codec_config.video = None;
        self
    }

    pub fn no_audio(mut self) -> Self {
        self.codec_config.audio = None;
        self
    }

    pub fn stun_server(mut self, url: impl Into<String>) -> Self {
        self.ice_config.stun_server = Some(url.into());
        self
    }

    pub fn add_turn_server(mut self, url: impl Into<String>) -> Self {
        self.ice_config.turn_servers.push(url.into());
        self
    }

    pub fn ice_config(mut self, config: IceServerConfig) -> Self {
        self.ice_config = config;
        self
    }

    /// Build the pipeline with the given media source.
    ///
    /// The pipeline is created in NULL state. Call `play()` to start streaming.
    pub fn build(self, source: &dyn MediaSource) -> Result<StreamPipeline> {
        let pipeline = match &self.pipeline_name {
            Some(name) => gst::Pipeline::with_name(name),
            None => gst::Pipeline::default(),
        };

        let decodebin = source.create_source_bin(&pipeline)?;

        let webrtcbin = make_element("webrtcbin", Some("webrtcbin"))?;
        if let Some(ref stun) = self.ice_config.stun_server {
            webrtcbin.set_property_from_str("stun-server", stun);
        }
        webrtcbin.set_property_from_str("bundle-policy", "max-bundle");

        pipeline
            .add(&webrtcbin)
            .map_err(|e| Error::PipelineConstruction(e.to_string()))?;

        let codec_config = self.codec_config.clone();
        let pipeline_weak = pipeline.downgrade();
        let webrtcbin_weak = webrtcbin.downgrade();
        let video_linked = Arc::new(AtomicBool::new(false));
        let audio_linked = Arc::new(AtomicBool::new(false));

        decodebin.connect_pad_added(move |_decodebin, src_pad| {
            let Some(pipeline) = pipeline_weak.upgrade() else {
                return;
            };
            let Some(webrtcbin) = webrtcbin_weak.upgrade() else {
                return;
            };

            let caps = src_pad
                .current_caps()
                .unwrap_or_else(|| src_pad.query_caps(None));
            let Some(structure) = caps.structure(0) else {
                return;
            };
            let media_type = structure.name().as_str();

            info!("decodebin pad-added: {media_type}");

            if media_type.starts_with("video/") {
                if video_linked.swap(true, Ordering::SeqCst) {
                    info!("Skipping additional video pad");
                    return;
                }
                if let Some(video_codec) = &codec_config.video {
                    if let Err(e) =
                        Self::link_video_branch(&pipeline, src_pad, video_codec, &webrtcbin)
                    {
                        tracing::error!("Failed to link video branch: {e}");
                        video_linked.store(false, Ordering::SeqCst);
                    }
                }
            } else if media_type.starts_with("audio/") {
                if audio_linked.swap(true, Ordering::SeqCst) {
                    info!("Skipping additional audio pad");
                    return;
                }
                if let Some(audio_codec) = &codec_config.audio {
                    if let Err(e) =
                        Self::link_audio_branch(&pipeline, src_pad, audio_codec, &webrtcbin)
                    {
                        tracing::error!("Failed to link audio branch: {e}");
                        audio_linked.store(false, Ordering::SeqCst);
                    }
                }
            }
        });

        let bus_events = spawn_bus_monitor(&pipeline);

        Ok(StreamPipeline {
            pipeline,
            webrtcbin,
            bus_events,
        })
    }

    fn link_video_branch(
        pipeline: &gst::Pipeline,
        src_pad: &gst::Pad,
        codec: &VideoCodec,
        webrtcbin: &gst::Element,
    ) -> Result<()> {
        let queue = make_element("queue", Some("video_queue"))?;
        let convert = make_element("videoconvert", Some("videoconvert"))?;
        let encoder = make_element(codec.encoder_element(), Some("video_encoder"))?;
        let payloader = make_element(codec.rtp_payloader(), Some("video_payloader"))?;

        match codec {
            VideoCodec::Vp8 => {
                encoder.set_property("deadline", 1i64);
                encoder.set_property_from_str("error-resilient", "partitions");
            }
            VideoCodec::H264 => {
                encoder.set_property_from_str("tune", "zerolatency");
                encoder.set_property_from_str("speed-preset", "ultrafast");
            }
            VideoCodec::Vp9 => {}
        }

        pipeline
            .add_many([&queue, &convert, &encoder, &payloader])
            .map_err(|e| Error::PipelineConstruction(e.to_string()))?;

        gst::Element::link_many([&queue, &convert, &encoder, &payloader]).map_err(|_| {
            Error::ElementLinkFailed {
                source_element: "video chain".into(),
                sink_element: "video chain".into(),
            }
        })?;

        payloader
            .link(webrtcbin)
            .map_err(|_| Error::ElementLinkFailed {
                source_element: "video_payloader".into(),
                sink_element: "webrtcbin".into(),
            })?;

        for el in [&queue, &convert, &encoder, &payloader] {
            el.sync_state_with_parent()
                .map_err(|e| Error::StateChange(e.to_string()))?;
        }

        let queue_sink = queue
            .static_pad("sink")
            .ok_or_else(|| Error::PipelineConstruction("queue has no sink pad".into()))?;
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
        codec: &AudioCodec,
        webrtcbin: &gst::Element,
    ) -> Result<()> {
        let queue = make_element("queue", Some("audio_queue"))?;
        let convert = make_element("audioconvert", Some("audioconvert"))?;
        let resample = make_element("audioresample", Some("audioresample"))?;
        let capsfilter = make_element("capsfilter", Some("audio_capsfilter"))?;
        let encoder = make_element(codec.encoder_element(), Some("audio_encoder"))?;
        let payloader = make_element(codec.rtp_payloader(), Some("audio_payloader"))?;

        // Force fixed caps so opusenc can negotiate properly
        let audio_caps =
            gst::Caps::builder("audio/x-raw")
                .field("rate", 48000i32)
                .field("channels", 2i32)
                .build();
        capsfilter.set_property("caps", &audio_caps);

        pipeline
            .add_many([&queue, &convert, &resample, &capsfilter, &encoder, &payloader])
            .map_err(|e| Error::PipelineConstruction(e.to_string()))?;

        gst::Element::link_many([&queue, &convert, &resample, &capsfilter, &encoder, &payloader])
            .map_err(|_| Error::ElementLinkFailed {
                source_element: "audio chain".into(),
                sink_element: "audio chain".into(),
            })?;

        payloader
            .link(webrtcbin)
            .map_err(|_| Error::ElementLinkFailed {
                source_element: "audio_payloader".into(),
                sink_element: "webrtcbin".into(),
            })?;

        for el in [&queue, &convert, &resample, &capsfilter, &encoder, &payloader] {
            el.sync_state_with_parent()
                .map_err(|e| Error::StateChange(e.to_string()))?;
        }

        let queue_sink = queue
            .static_pad("sink")
            .ok_or_else(|| Error::PipelineConstruction("queue has no sink pad".into()))?;
        src_pad
            .link(&queue_sink)
            .map_err(|_| Error::ElementLinkFailed {
                source_element: "decodebin".into(),
                sink_element: "audio_queue".into(),
            })?;

        Ok(())
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
