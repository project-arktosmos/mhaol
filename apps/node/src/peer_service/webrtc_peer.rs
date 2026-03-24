use super::types::{DataChannelEnvelope, IceServerConfig};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, info, warn};
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;

/// Events emitted by a WebRTC peer connection.
#[derive(Debug)]
pub enum PeerEvent {
    /// Data channel opened and ready.
    ChannelOpen { peer_id: String },
    /// Received a message on the data channel.
    Message {
        peer_id: String,
        envelope: DataChannelEnvelope,
    },
    /// Peer connection closed or failed.
    Disconnected { peer_id: String },
    /// ICE candidate generated locally, needs to be sent via signaling.
    IceCandidate {
        peer_id: String,
        candidate: String,
        sdp_mline_index: u32,
        sdp_mid: Option<String>,
    },
    /// SDP answer generated (when answering an offer).
    Answer { peer_id: String, sdp: String },
}

/// Manages a single WebRTC peer connection with a data channel.
pub struct WebRtcPeer {
    pub peer_id: String,
    connection: Arc<RTCPeerConnection>,
    data_channel: Arc<Mutex<Option<Arc<RTCDataChannel>>>>,
}

impl WebRtcPeer {
    /// Create a new WebRTC peer connection that answers an incoming offer.
    ///
    /// Creates the peer connection, sets the remote description (offer),
    /// creates an answer, and starts ICE gathering. Events are sent via `event_tx`.
    pub async fn answer_offer(
        peer_id: String,
        offer_sdp: &str,
        ice_servers: &[IceServerConfig],
        event_tx: mpsc::UnboundedSender<PeerEvent>,
    ) -> Result<Self, String> {
        let api = create_api()?;
        let config = create_config(ice_servers);

        let connection = api
            .new_peer_connection(config)
            .await
            .map_err(|e| format!("Failed to create peer connection: {e}"))?;
        let connection = Arc::new(connection);
        let data_channel: Arc<Mutex<Option<Arc<RTCDataChannel>>>> = Arc::new(Mutex::new(None));

        // Set up event handlers
        setup_connection_handlers(
            &connection,
            &peer_id,
            &event_tx,
            Arc::clone(&data_channel),
        );

        // Set remote description (the offer from the browser peer)
        let offer = RTCSessionDescription::offer(offer_sdp.to_string())
            .map_err(|e| format!("Invalid offer SDP: {e}"))?;
        connection
            .set_remote_description(offer)
            .await
            .map_err(|e| format!("Failed to set remote description: {e}"))?;

        // Create and set local description (answer)
        let answer = connection
            .create_answer(None)
            .await
            .map_err(|e| format!("Failed to create answer: {e}"))?;

        let answer_sdp = answer.sdp.clone();
        connection
            .set_local_description(answer)
            .await
            .map_err(|e| format!("Failed to set local description: {e}"))?;

        // Send the answer SDP back via events
        let _ = event_tx.send(PeerEvent::Answer {
            peer_id: peer_id.clone(),
            sdp: answer_sdp,
        });

        Ok(Self {
            peer_id,
            connection,
            data_channel,
        })
    }

    /// Add an ICE candidate received from the remote peer.
    pub async fn add_ice_candidate(
        &self,
        candidate: &str,
        sdp_mline_index: u32,
        sdp_mid: Option<&str>,
    ) -> Result<(), String> {
        let init = RTCIceCandidateInit {
            candidate: candidate.to_string(),
            sdp_mline_index: Some(sdp_mline_index as u16),
            sdp_mid: sdp_mid.map(String::from),
            ..Default::default()
        };
        self.connection
            .add_ice_candidate(init)
            .await
            .map_err(|e| format!("Failed to add ICE candidate: {e}"))
    }

    /// Send a data channel envelope to the remote peer.
    pub async fn send(&self, envelope: &DataChannelEnvelope) -> Result<(), String> {
        let guard = self.data_channel.lock().await;
        let dc = guard
            .as_ref()
            .ok_or_else(|| "Data channel not open".to_string())?;

        let json = serde_json::to_string(envelope)
            .map_err(|e| format!("Failed to serialize envelope: {e}"))?;
        dc.send_text(json)
            .await
            .map(|_| ())
            .map_err(|e| format!("Failed to send on data channel: {e}"))
    }

    /// Close the peer connection.
    pub async fn close(&self) {
        if let Err(e) = self.connection.close().await {
            warn!(peer_id = %self.peer_id, "Error closing peer connection: {e}");
        }
    }
}

/// Set up event handlers for a peer connection.
fn setup_connection_handlers(
    connection: &Arc<RTCPeerConnection>,
    peer_id: &str,
    event_tx: &mpsc::UnboundedSender<PeerEvent>,
    data_channel: Arc<Mutex<Option<Arc<RTCDataChannel>>>>,
) {
    // ICE candidate handler
    let peer_id_ice = peer_id.to_string();
    let event_tx_ice = event_tx.clone();
    connection.on_ice_candidate(Box::new(move |candidate| {
        let peer_id = peer_id_ice.clone();
        let event_tx = event_tx_ice.clone();
        Box::pin(async move {
            if let Some(c) = candidate {
                let json = match c.to_json() {
                    Ok(j) => j,
                    Err(e) => {
                        warn!("Failed to serialize ICE candidate: {e}");
                        return;
                    }
                };
                let _ = event_tx.send(PeerEvent::IceCandidate {
                    peer_id,
                    candidate: json.candidate,
                    sdp_mline_index: json.sdp_mline_index.unwrap_or(0) as u32,
                    sdp_mid: json.sdp_mid,
                });
            }
        })
    }));

    // Connection state handler
    let peer_id_state = peer_id.to_string();
    let event_tx_state = event_tx.clone();
    connection.on_peer_connection_state_change(Box::new(move |state| {
        let peer_id = peer_id_state.clone();
        let event_tx = event_tx_state.clone();
        Box::pin(async move {
            info!(peer_id = %peer_id, state = ?state, "Peer connection state changed");
            match state {
                RTCPeerConnectionState::Failed
                | RTCPeerConnectionState::Disconnected
                | RTCPeerConnectionState::Closed => {
                    let _ = event_tx.send(PeerEvent::Disconnected { peer_id });
                }
                _ => {}
            }
        })
    }));

    // Data channel handler (for channels created by the remote peer)
    let peer_id_dc = peer_id.to_string();
    let event_tx_dc = event_tx.clone();
    let data_channel_dc = data_channel;
    connection.on_data_channel(Box::new(move |dc| {
        let peer_id = peer_id_dc.clone();
        let event_tx = event_tx_dc.clone();
        let data_channel = data_channel_dc.clone();

        Box::pin(async move {
            let label = dc.label().to_string();
            debug!(peer_id = %peer_id, label = %label, "Data channel received");

            let dc = Arc::new(dc);

            // Store the data channel reference
            {
                let mut guard = data_channel.lock().await;
                *guard = Some(Arc::clone(&dc));
            }

            // On open
            let peer_id_open = peer_id.clone();
            let event_tx_open = event_tx.clone();
            dc.on_open(Box::new(move || {
                let peer_id = peer_id_open.clone();
                let event_tx = event_tx_open.clone();
                Box::pin(async move {
                    info!(peer_id = %peer_id, "Data channel open");
                    let _ = event_tx.send(PeerEvent::ChannelOpen { peer_id });
                })
            }));

            // On message
            let peer_id_msg = peer_id.clone();
            let event_tx_msg = event_tx.clone();
            dc.on_message(Box::new(move |msg: DataChannelMessage| {
                let peer_id = peer_id_msg.clone();
                let event_tx = event_tx_msg.clone();
                Box::pin(async move {
                    let text = match String::from_utf8(msg.data.to_vec()) {
                        Ok(t) => t,
                        Err(_) => return,
                    };
                    match serde_json::from_str::<DataChannelEnvelope>(&text) {
                        Ok(envelope) => {
                            let _ = event_tx.send(PeerEvent::Message { peer_id, envelope });
                        }
                        Err(e) => {
                            debug!(peer_id = %peer_id, "Non-envelope data channel message: {e}");
                        }
                    }
                })
            }));
        })
    }));
}

/// Manages multiple WebRTC peer connections.
pub struct PeerManager {
    peers: HashMap<String, WebRtcPeer>,
    ice_servers: Vec<IceServerConfig>,
    pub event_tx: mpsc::UnboundedSender<PeerEvent>,
    pub event_rx: Option<mpsc::UnboundedReceiver<PeerEvent>>,
}

impl PeerManager {
    pub fn new() -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        Self {
            peers: HashMap::new(),
            ice_servers: Vec::new(),
            event_tx,
            event_rx: Some(event_rx),
        }
    }

    /// Take the event receiver (can only be called once).
    pub fn take_event_rx(&mut self) -> Option<mpsc::UnboundedReceiver<PeerEvent>> {
        self.event_rx.take()
    }

    /// Update ICE servers (received from PartyKit's `connected` message).
    pub fn set_ice_servers(&mut self, servers: Vec<IceServerConfig>) {
        self.ice_servers = servers;
    }

    /// Handle an incoming SDP offer: create a peer and answer it.
    pub async fn handle_offer(
        &mut self,
        peer_id: &str,
        sdp: &str,
    ) -> Result<(), String> {
        // Remove any existing peer connection for this peer
        if let Some(old) = self.peers.remove(peer_id) {
            old.close().await;
        }

        let peer = WebRtcPeer::answer_offer(
            peer_id.to_string(),
            sdp,
            &self.ice_servers,
            self.event_tx.clone(),
        )
        .await?;

        self.peers.insert(peer_id.to_string(), peer);
        Ok(())
    }

    /// Handle an incoming ICE candidate for a peer.
    pub async fn handle_ice_candidate(
        &self,
        peer_id: &str,
        candidate: &str,
        sdp_mline_index: u32,
        sdp_mid: Option<&str>,
    ) -> Result<(), String> {
        let peer = self
            .peers
            .get(peer_id)
            .ok_or_else(|| format!("No peer connection for {peer_id}"))?;
        peer.add_ice_candidate(candidate, sdp_mline_index, sdp_mid)
            .await
    }

    /// Send a data channel envelope to a specific peer.
    pub async fn send_to_peer(
        &self,
        peer_id: &str,
        envelope: &DataChannelEnvelope,
    ) -> Result<(), String> {
        let peer = self
            .peers
            .get(peer_id)
            .ok_or_else(|| format!("No peer connection for {peer_id}"))?;
        peer.send(envelope).await
    }

    /// Remove and close a peer connection.
    pub async fn remove_peer(&mut self, peer_id: &str) {
        if let Some(peer) = self.peers.remove(peer_id) {
            peer.close().await;
        }
    }

    /// Check if a peer has an active connection.
    pub fn has_peer(&self, peer_id: &str) -> bool {
        self.peers.contains_key(peer_id)
    }
}

fn create_api() -> Result<webrtc::api::API, String> {
    let mut media_engine = MediaEngine::default();
    media_engine
        .register_default_codecs()
        .map_err(|e| format!("Failed to register codecs: {e}"))?;

    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut media_engine)
        .map_err(|e| format!("Failed to register interceptors: {e}"))?;

    Ok(APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .build())
}

fn create_config(ice_servers: &[IceServerConfig]) -> RTCConfiguration {
    let mut servers: Vec<RTCIceServer> = ice_servers
        .iter()
        .map(|s| RTCIceServer {
            urls: s.urls.clone(),
            username: s.username.clone().unwrap_or_default(),
            credential: s.credential.clone().unwrap_or_default(),
            ..Default::default()
        })
        .collect();

    // Always include a default STUN server
    if servers.is_empty() {
        servers.push(RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_string()],
            ..Default::default()
        });
    }

    RTCConfiguration {
        ice_servers: servers,
        ..Default::default()
    }
}
