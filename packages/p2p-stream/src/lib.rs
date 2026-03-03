pub mod error;
pub mod media;
pub mod pipeline;
pub mod session;
pub mod signaling;
pub mod worker;

use std::sync::Once;

static GST_INIT: Once = Once::new();

/// Initialize GStreamer. Safe to call multiple times; only executes once.
/// Must be called before any other p2p-stream API.
pub fn init() -> Result<(), error::Error> {
    let mut result = Ok(());
    GST_INIT.call_once(|| {
        if let Err(e) = gstreamer::init() {
            result = Err(error::Error::GstreamerInit(e.to_string()));
        }
    });
    result
}

/// Required GStreamer elements for streaming to work.
const REQUIRED_ELEMENTS: &[&str] = &[
    "filesrc",
    "decodebin",
    "videoconvert",
    "audioconvert",
    "audioresample",
    "queue",
    "opusenc",
    "rtpopuspay",
    "webrtcbin",
];

/// Check that all required GStreamer elements are available.
/// Returns a list of missing element factory names (empty if all present).
/// GStreamer must be initialized first via [`init`].
pub fn check_required_elements() -> Vec<&'static str> {
    REQUIRED_ELEMENTS
        .iter()
        .filter(|name| gstreamer::ElementFactory::find(name).is_none())
        .copied()
        .collect()
}

pub mod prelude {
    pub use crate::error::Error;
    pub use crate::media::{AudioCodec, CodecConfig, FileSource, MediaSource, VideoCodec, VideoQuality};
    pub use crate::pipeline::{BusEvent, PipelineBuilder, StreamPipeline};
    pub use crate::session::{PeerSession, SessionManager, SessionState};
    pub use crate::signaling::{IceCandidate, SessionDescription, SignalingMessage};
}
