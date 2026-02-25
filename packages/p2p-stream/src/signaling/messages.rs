use serde::{Deserialize, Serialize};

/// Unique identifier for a peer session.
pub type PeerId = String;

/// SDP type: offer or answer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SdpType {
    Offer,
    Answer,
}

/// SDP session description exchanged during WebRTC negotiation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDescription {
    pub sdp_type: SdpType,
    pub sdp: String,
}

/// ICE candidate exchanged during connectivity checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceCandidate {
    pub sdp_m_line_index: u32,
    pub candidate: String,
}

/// Seek command from the browser.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeekCommand {
    pub position_secs: f64,
}

/// One-time media info sent when the pipeline discovers media metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfoPayload {
    pub duration_secs: Option<f64>,
}

/// Periodic position update from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionPayload {
    pub position_secs: f64,
    pub duration_secs: Option<f64>,
}

/// Messages that flow through the signaling channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum SignalingMessage {
    SessionDescription(SessionDescription),
    IceCandidate(IceCandidate),
    IceGatheringComplete,
    PeerDisconnected { peer_id: PeerId },
    Seek(SeekCommand),
    MediaInfo(MediaInfoPayload),
    PositionUpdate(PositionPayload),
}
