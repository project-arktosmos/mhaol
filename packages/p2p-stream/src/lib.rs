pub mod error;
pub mod media;
pub mod pipeline;
pub mod session;
pub mod signaling;

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

pub mod prelude {
    pub use crate::error::Error;
    pub use crate::media::{CodecConfig, FileSource, MediaSource, VideoCodec, AudioCodec};
    pub use crate::pipeline::{BusEvent, PipelineBuilder, StreamPipeline};
    pub use crate::session::{PeerSession, SessionManager, SessionState};
    pub use crate::signaling::{IceCandidate, SessionDescription, SignalingMessage};
}
