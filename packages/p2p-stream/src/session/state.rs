use serde::{Deserialize, Serialize};

/// The lifecycle state of a WebRTC peer session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// Session created, pipeline not yet started.
    New,
    /// Pipeline is playing, awaiting negotiation.
    Connecting,
    /// SDP offer has been created and sent.
    OfferSent,
    /// SDP answer has been received and applied.
    Connected,
    /// ICE connection has been established.
    Streaming,
    /// Session is being torn down.
    Disconnecting,
    /// Session has been fully cleaned up.
    Closed,
    /// An error occurred.
    Failed,
}
