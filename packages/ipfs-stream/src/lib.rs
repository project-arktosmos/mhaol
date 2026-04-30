//! HLS-over-IPFS streaming for Mhaol.
//!
//! This crate transcodes a locally pinned IPFS file into HTTP Live Streaming
//! segments (`playlist.m3u8` + `segment*.ts`) on demand. The cloud server is
//! expected to serve those files over HTTP so a browser-side `hls.js` player
//! can consume them.
//!
//! Pipeline topology:
//!
//! ```text
//! filesrc -> decodebin -+-> videoconvert -> x264enc ----+-> mpegtsmux -> hlssink2
//!                       \                                /
//!                        +-> audioconvert -> aacenc ----+
//! ```
//!
//! The "source" of every stream is a CID that the cloud has already pinned
//! locally (i.e. the file is on disk). Conceptually the bytes flow from the
//! IPFS swarm into the local repo, then through this crate, out as HLS.

pub mod error;
pub mod manager;
pub mod pipeline;
pub mod session;

use std::sync::Once;

static GST_INIT: Once = Once::new();

/// Initialize GStreamer. Safe to call multiple times; only executes once.
/// Must be called before any other ipfs-stream API.
pub fn init() -> Result<(), error::Error> {
    let mut result = Ok(());
    GST_INIT.call_once(|| {
        if let Err(e) = gstreamer::init() {
            result = Err(error::Error::GstreamerInit(e.to_string()));
        }
    });
    result
}

/// GStreamer elements this crate needs to build a working HLS pipeline.
/// At least one of the AAC encoders has to be present for the audio
/// branch to link.
const REQUIRED_ELEMENTS: &[&str] = &[
    "filesrc",
    "decodebin",
    "videoconvert",
    "x264enc",
    "h264parse",
    "audioconvert",
    "audioresample",
    "aacparse",
    "mpegtsmux",
    "hlssink2",
];

const REQUIRED_AAC_ENCODERS: &[&str] = &["avenc_aac", "voaacenc", "faac"];

/// Check that the required GStreamer elements are available.
/// Returns the list of missing factory names (empty if all present).
/// GStreamer must be initialized first via [`init`].
pub fn check_required_elements() -> Vec<&'static str> {
    let mut missing: Vec<&'static str> = REQUIRED_ELEMENTS
        .iter()
        .filter(|name| gstreamer::ElementFactory::find(name).is_none())
        .copied()
        .collect();
    let has_any_aac = REQUIRED_AAC_ENCODERS
        .iter()
        .any(|name| gstreamer::ElementFactory::find(name).is_some());
    if !has_any_aac {
        missing.push("avenc_aac|voaacenc|faac");
    }
    missing
}

pub mod prelude {
    pub use crate::error::Error;
    pub use crate::manager::{IpfsStreamManager, StartedSession};
    pub use crate::pipeline::{HlsPipeline, HlsPipelineConfig};
    pub use crate::session::{SessionInfo, SessionState};
}
