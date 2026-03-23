use crate::error::{Error, Result};
use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use std::io::Read;
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};

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

/// An HTTP-based media source using appsrc.
///
/// Pipeline: `appsrc → decodebin`
/// A dedicated thread reads from the HTTP URL using a blocking client (ureq)
/// and pushes data into appsrc. This handles librqbit's streaming endpoint
/// correctly — when a torrent piece isn't available yet, the HTTP body blocks
/// and the thread waits until data arrives.
pub struct AppSrcSource {
    url: String,
    has_video: bool,
    has_audio: bool,
}

impl AppSrcSource {
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

impl MediaSource for AppSrcSource {
    fn create_source_bin(&self, pipeline: &gst::Pipeline) -> Result<gst::Element> {
        let appsrc = gst::ElementFactory::make("appsrc")
            .name("appsrc")
            .property("is-live", true)
            .property("format", gst::Format::Bytes)
            .property(
                "stream-type",
                gst_app::AppStreamType::Stream,
            )
            .property("block", true)
            .build()
            .map_err(|e| Error::ElementNotFound(format!("appsrc: {e}")))?;

        let decodebin = gst::ElementFactory::make("decodebin")
            .name("decodebin")
            .build()
            .map_err(|e| Error::ElementNotFound(format!("decodebin: {e}")))?;

        pipeline
            .add_many([&appsrc, &decodebin])
            .map_err(|e| Error::PipelineConstruction(e.to_string()))?;

        appsrc.link(&decodebin).map_err(|_| Error::ElementLinkFailed {
            source_element: "appsrc".into(),
            sink_element: "decodebin".into(),
        })?;

        let url = self.url.clone();
        let appsrc_weak = appsrc.downgrade();

        std::thread::Builder::new()
            .name("appsrc-feeder".into())
            .spawn(move || {
                feed_from_url(&url, &appsrc_weak);
            })
            .map_err(|e| Error::PipelineConstruction(format!("Failed to spawn feeder thread: {e}")))?;

        Ok(decodebin)
    }

    fn has_video(&self) -> bool {
        self.has_video
    }

    fn has_audio(&self) -> bool {
        self.has_audio
    }
}

/// Read from an HTTP URL in a blocking loop and push data into appsrc.
///
/// Retries the connection on errors with a 2-second delay (up to 30 attempts).
/// The blocking `read()` call naturally waits when librqbit's torrent stream
/// is waiting for pieces to download.
fn feed_from_url(url: &str, appsrc_weak: &gst::glib::WeakRef<gst::Element>) {
    const CHUNK_SIZE: usize = 65536;
    const MAX_RETRIES: u32 = 30;
    const RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(2);

    let mut attempt = 0u32;

    loop {
        let appsrc = match appsrc_weak.upgrade() {
            Some(el) => el,
            None => {
                info!("appsrc dropped, feeder stopping");
                return;
            }
        };
        let appsrc = appsrc.downcast_ref::<gst_app::AppSrc>().unwrap();

        info!("Connecting to stream URL (attempt {})", attempt + 1);

        let response = match ureq::get(url).call() {
            Ok(resp) => resp,
            Err(e) => {
                attempt += 1;
                if attempt >= MAX_RETRIES {
                    error!("Failed to connect after {MAX_RETRIES} attempts: {e}");
                    let _ = appsrc.end_of_stream();
                    return;
                }
                warn!("HTTP connect error (attempt {attempt}/{MAX_RETRIES}): {e}");
                std::thread::sleep(RETRY_DELAY);
                continue;
            }
        };

        info!("Connected to stream, reading data");
        let mut reader = response.into_body().into_reader();
        let mut buf = vec![0u8; CHUNK_SIZE];

        loop {
            let appsrc = match appsrc_weak.upgrade() {
                Some(el) => el,
                None => {
                    info!("appsrc dropped during read, feeder stopping");
                    return;
                }
            };
            let appsrc = appsrc.downcast_ref::<gst_app::AppSrc>().unwrap();

            match reader.read(&mut buf) {
                Ok(0) => {
                    info!("Stream EOF, sending EOS");
                    let _ = appsrc.end_of_stream();
                    return;
                }
                Ok(n) => {
                    let mut gst_buf = gst::Buffer::with_size(n).unwrap();
                    {
                        let buf_ref = gst_buf.get_mut().unwrap();
                        let mut map = buf_ref.map_writable().unwrap();
                        map[..n].copy_from_slice(&buf[..n]);
                    }

                    match appsrc.push_buffer(gst_buf) {
                        Ok(_) => {
                            // Reset retry counter on successful data flow
                            attempt = 0;
                        }
                        Err(gst::FlowError::Eos) => {
                            info!("appsrc returned EOS, feeder stopping");
                            return;
                        }
                        Err(e) => {
                            info!("appsrc push error ({e:?}), pipeline likely stopped");
                            return;
                        }
                    }
                }
                Err(e) => {
                    warn!("Stream read error: {e}");
                    // Break inner loop to retry the connection
                    break;
                }
            }
        }

        // Retry after read error
        attempt += 1;
        if attempt >= MAX_RETRIES {
            error!("Too many read errors, giving up after {MAX_RETRIES} attempts");
            if let Some(el) = appsrc_weak.upgrade() {
                let appsrc = el.downcast_ref::<gst_app::AppSrc>().unwrap();
                let _ = appsrc.end_of_stream();
            }
            return;
        }
        warn!("Retrying connection in {}s (attempt {attempt}/{MAX_RETRIES})", RETRY_DELAY.as_secs());
        std::thread::sleep(RETRY_DELAY);
    }
}
