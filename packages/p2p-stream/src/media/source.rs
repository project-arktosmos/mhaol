use crate::error::{Error, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use std::path::{Path, PathBuf};

/// Blanket impl so `Box<dyn MediaSource>` can be passed where `impl MediaSource` is expected.
impl MediaSource for Box<dyn MediaSource> {
    fn create_source_bin(&self, pipeline: &gst::Pipeline) -> Result<gst::Element> {
        (**self).create_source_bin(pipeline)
    }
    fn has_video(&self) -> bool {
        (**self).has_video()
    }
    fn has_audio(&self) -> bool {
        (**self).has_audio()
    }
}

/// Describes a media source that can be plugged into the pipeline.
pub trait MediaSource: Send + Sync {
    /// Create the GStreamer source elements and add them to the pipeline.
    /// Returns the element whose pads will be connected downstream (typically decodebin).
    fn create_source_bin(&self, pipeline: &gst::Pipeline) -> Result<gst::Element>;

    /// Whether this source produces video.
    fn has_video(&self) -> bool;

    /// Whether this source produces audio.
    fn has_audio(&self) -> bool;
}

/// A file-based media source.
///
/// Pipeline: `filesrc → decodebin` (auto-detects container and codec).
/// decodebin dynamically creates pads as it discovers streams.
pub struct FileSource {
    path: PathBuf,
    has_video: bool,
    has_audio: bool,
}

impl FileSource {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            has_video: true,
            has_audio: true,
        }
    }

    pub fn video_only(mut self) -> Self {
        self.has_audio = false;
        self
    }

    pub fn audio_only(mut self) -> Self {
        self.has_video = false;
        self
    }
}

impl MediaSource for FileSource {
    fn create_source_bin(&self, pipeline: &gst::Pipeline) -> Result<gst::Element> {
        if !self.path.exists() {
            return Err(Error::MediaSource(format!(
                "File not found: {}",
                self.path.display()
            )));
        }

        let filesrc = gst::ElementFactory::make("filesrc")
            .name("filesrc")
            .property("location", self.path.to_str().unwrap_or_default())
            .build()
            .map_err(|e| Error::ElementNotFound(format!("filesrc: {e}")))?;

        let decodebin = gst::ElementFactory::make("decodebin")
            .name("decodebin")
            .build()
            .map_err(|e| Error::ElementNotFound(format!("decodebin: {e}")))?;

        pipeline
            .add_many([&filesrc, &decodebin])
            .map_err(|e| Error::PipelineConstruction(e.to_string()))?;

        filesrc.link(&decodebin).map_err(|_| Error::ElementLinkFailed {
            source_element: "filesrc".into(),
            sink_element: "decodebin".into(),
        })?;

        Ok(decodebin)
    }

    fn has_video(&self) -> bool {
        self.has_video
    }

    fn has_audio(&self) -> bool {
        self.has_audio
    }
}

/// An HTTP-based media source.
///
/// Pipeline: `souphttpsrc → decodebin` (reads media over HTTP with Range request support).
/// Used to stream from librqbit's HTTP API for torrent playback via WebRTC.
pub struct HttpSource {
    url: String,
    has_video: bool,
    has_audio: bool,
}

impl HttpSource {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            has_video: true,
            has_audio: true,
        }
    }

    pub fn video_only(mut self) -> Self {
        self.has_audio = false;
        self
    }

    pub fn audio_only(mut self) -> Self {
        self.has_video = false;
        self
    }
}

impl MediaSource for HttpSource {
    fn create_source_bin(&self, pipeline: &gst::Pipeline) -> Result<gst::Element> {
        let httpsrc = gst::ElementFactory::make("souphttpsrc")
            .name("httpsrc")
            .property("location", &self.url)
            // is-live=true: treat as a live source so GStreamer tolerates
            // buffering pauses when librqbit is waiting for torrent pieces.
            // This disables pull-mode seeking but we handle seek via the
            // data channel → pipeline.seek_simple() path instead.
            .property("is-live", true)
            // timeout=0: unlimited — librqbit blocks until pieces arrive,
            // which can take arbitrarily long for slow torrents.
            .property("timeout", 0u32)
            .property("retries", -1i32)
            .build()
            .map_err(|e| Error::ElementNotFound(format!("souphttpsrc: {e}")))?;

        let decodebin = gst::ElementFactory::make("decodebin")
            .name("decodebin")
            .build()
            .map_err(|e| Error::ElementNotFound(format!("decodebin: {e}")))?;

        pipeline
            .add_many([&httpsrc, &decodebin])
            .map_err(|e| Error::PipelineConstruction(e.to_string()))?;

        httpsrc.link(&decodebin).map_err(|_| Error::ElementLinkFailed {
            source_element: "httpsrc".into(),
            sink_element: "decodebin".into(),
        })?;

        Ok(decodebin)
    }

    fn has_video(&self) -> bool {
        self.has_video
    }

    fn has_audio(&self) -> bool {
        self.has_audio
    }
}
