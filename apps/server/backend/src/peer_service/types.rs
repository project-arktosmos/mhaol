use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ===== Passport & Identity =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassportData {
    pub raw: String,
    #[serde(default)]
    pub hash: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PassportPayload {
    pub name: String,
    pub address: String,
    pub instance_type: String,
    pub signaling_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Endorsement {
    pub passport_raw: String,
    pub endorser_signature: String,
    pub endorser_address: String,
    pub endorsed_at: String,
}

// ===== Data Channel Envelope =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataChannelEnvelope {
    pub channel: String,
    pub payload: serde_json::Value,
}

// ===== Contact Handshake Messages =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ContactMessage {
    ContactRequest { passport: PassportData },
    ContactAccept {
        passport: PassportData,
        #[serde(skip_serializing_if = "Option::is_none")]
        endorsement: Option<Endorsement>,
    },
}

// ===== Server Catalog Messages =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ServerCatalogMessage {
    CatalogMovies {
        movies: Vec<CatalogMovie>,
    },
    StreamRequest {
        #[serde(rename = "tmdbId")]
        tmdb_id: i64,
    },
    StreamSession {
        #[serde(rename = "sessionId")]
        session_id: String,
        #[serde(rename = "roomId")]
        room_id: String,
        #[serde(rename = "signalingUrl")]
        signaling_url: String,
    },
    StreamError {
        error: String,
    },
    CatalogRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogMovie {
    pub item: CatalogMediaItem,
    pub tmdb: Option<serde_json::Value>,
    pub streamable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogMediaItem {
    pub id: String,
    pub library_id: String,
    pub name: String,
    #[serde(default)]
    pub extension: String,
    pub path: String,
    #[serde(default)]
    pub category_id: Option<String>,
    #[serde(default)]
    pub media_type_id: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub links: HashMap<String, CatalogMediaItemLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogMediaItemLink {
    pub service_id: String,
    #[serde(default)]
    pub service_url: Option<String>,
}

// ===== Signaling Server Messages =====

#[derive(Debug, Clone)]
pub struct IceServerConfig {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: String,
    pub name: String,
    pub instance_type: String,
}

#[derive(Debug, Clone)]
pub enum SignalingServerMessage {
    Connected {
        peer_id: String,
        ice_servers: Vec<IceServerConfig>,
    },
    RoomPeers {
        peers: Vec<PeerInfo>,
    },
    PeerJoined {
        peer_id: String,
        name: String,
        instance_type: String,
    },
    PeerLeft {
        peer_id: String,
    },
    Offer {
        from_peer_id: String,
        sdp: String,
    },
    Answer {
        from_peer_id: String,
        sdp: String,
    },
    IceCandidate {
        from_peer_id: String,
        candidate: String,
        sdp_m_line_index: u32,
        sdp_mid: Option<String>,
    },
    Error {
        message: String,
    },
}

/// Parse a raw JSON message from the PartyKit signaling server.
pub fn parse_signaling_message(text: &str) -> Option<SignalingServerMessage> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    let msg_type = value.get("type")?.as_str()?;

    match msg_type {
        "connected" => {
            let peer_id = value.get("peer_id")?.as_str()?.to_string();
            let ice_servers = value
                .get("ice_servers")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|s| {
                            let urls = s.get("urls").and_then(|u| {
                                if let Some(arr) = u.as_array() {
                                    Some(
                                        arr.iter()
                                            .filter_map(|v| v.as_str().map(String::from))
                                            .collect(),
                                    )
                                } else {
                                    u.as_str().map(|s| vec![s.to_string()])
                                }
                            })?;
                            Some(IceServerConfig {
                                urls,
                                username: s
                                    .get("username")
                                    .and_then(|v| v.as_str())
                                    .map(String::from),
                                credential: s
                                    .get("credential")
                                    .and_then(|v| v.as_str())
                                    .map(String::from),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            Some(SignalingServerMessage::Connected {
                peer_id,
                ice_servers,
            })
        }
        "room-peers" => {
            let peers = value
                .get("peers")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|p| {
                            Some(PeerInfo {
                                peer_id: p.get("peer_id")?.as_str()?.to_string(),
                                name: p
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                instance_type: p
                                    .get("instance_type")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
            Some(SignalingServerMessage::RoomPeers { peers })
        }
        "peer-joined" => Some(SignalingServerMessage::PeerJoined {
            peer_id: value.get("peer_id")?.as_str()?.to_string(),
            name: value
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            instance_type: value
                .get("instance_type")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }),
        "peer-left" => Some(SignalingServerMessage::PeerLeft {
            peer_id: value.get("peer_id")?.as_str()?.to_string(),
        }),
        "offer" => Some(SignalingServerMessage::Offer {
            from_peer_id: value.get("from_peer_id")?.as_str()?.to_string(),
            sdp: value.get("sdp")?.as_str()?.to_string(),
        }),
        "answer" => Some(SignalingServerMessage::Answer {
            from_peer_id: value.get("from_peer_id")?.as_str()?.to_string(),
            sdp: value.get("sdp")?.as_str()?.to_string(),
        }),
        "ice-candidate" => Some(SignalingServerMessage::IceCandidate {
            from_peer_id: value.get("from_peer_id")?.as_str()?.to_string(),
            candidate: value.get("candidate")?.as_str()?.to_string(),
            sdp_m_line_index: value
                .get("sdp_m_line_index")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            sdp_mid: value
                .get("sdp_mid")
                .and_then(|v| v.as_str())
                .map(String::from),
        }),
        "error" => Some(SignalingServerMessage::Error {
            message: value
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
        }),
        _ => None,
    }
}
