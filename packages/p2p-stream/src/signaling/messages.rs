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

/// Messages that flow through the signaling channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum SignalingMessage {
    SessionDescription(SessionDescription),
    IceCandidate(IceCandidate),
    IceGatheringComplete,
    PeerDisconnected { peer_id: PeerId },
}
