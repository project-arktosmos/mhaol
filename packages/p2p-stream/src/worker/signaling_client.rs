use futures_util::{SinkExt, StreamExt};
use k256::ecdsa::SigningKey;
use mhaol_p2p_stream::prelude::*;
use mhaol_p2p_stream::signaling::SdpType;
// Note: `mhaol_p2p_stream` is this crate's own lib — Cargo resolves `[[bin]]`
// targets' `extern crate` references to the same package's `[lib]`.
use sha3::{Digest, Keccak256};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::tungstenite;
use tracing::{debug, error, info, warn};

/// A live session entry tracked by the signaling client.
struct SessionEntry {
    manager: SessionManager,
    /// Tracks which peers have been created (receiver already moved to forwarder).
    active_peers: HashMap<String, ()>,
    created_at: Instant,
}

/// Connects to a PartyKit signaling room as a WebSocket client.
///
/// Handles the full lifecycle:
/// - Generates an ephemeral secp256k1 keypair for auth
/// - Connects to the PartyKit room with Ethereum-signed credentials
/// - When a browser peer joins, creates a PeerSession in the SessionManager
/// - Relays SDP/ICE between p2p-stream and PartyKit
/// - When a browser peer leaves, removes the PeerSession
pub struct SignalingClient {
    session_id: String,
    manager: Arc<RwLock<Option<SessionEntry>>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl SignalingClient {
    /// Create a signaling client and connect to the PartyKit room.
    ///
    /// `signaling_url` is the base URL like `http://localhost:1999` or
    /// `https://myapp.partykit.dev`. The room ID is the session_id.
    pub async fn connect(
        session_id: String,
        manager: SessionManager,
        signaling_url: &str,
    ) -> Result<Self, String> {
        let signing_key = SigningKey::random(&mut k256::elliptic_curve::rand_core::OsRng);
        let address = eth_address_from_key(&signing_key);
        let room_id = &session_id;

        let ws_url = build_ws_url(signaling_url, room_id, &signing_key, &address)?;

        info!(
            session_id = %session_id,
            address = %address,
            "Connecting to signaling room"
        );

        let (ws_stream, _response) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .map_err(|e| format!("WebSocket connect failed: {e}"))?;

        info!(session_id = %session_id, "Connected to signaling room");

        let entry = SessionEntry {
            manager,
            active_peers: HashMap::new(),
            created_at: Instant::now(),
        };
        let manager_arc = Arc::new(RwLock::new(Some(entry)));

        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);

        // Spawn the WebSocket message loop
        let manager_for_task = manager_arc.clone();
        let session_id_clone = session_id.clone();
        tokio::spawn(async move {
            run_ws_loop(ws_stream, manager_for_task, session_id_clone, shutdown_rx).await;
        });

        Ok(Self {
            session_id,
            manager: manager_arc,
            shutdown_tx: Some(shutdown_tx),
        })
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Disconnect from the signaling room and clean up all peer sessions.
    pub async fn disconnect(&mut self) {
        // Signal shutdown
        self.shutdown_tx.take();

        // Drop all peer sessions
        if let Some(entry) = self.manager.write().await.take() {
            for peer_id in entry.active_peers.keys() {
                let _ = entry.manager.remove_session(peer_id);
            }
        }

        info!(session_id = %self.session_id, "Disconnected from signaling room");
    }
}

/// Main WebSocket event loop.
async fn run_ws_loop(
    ws_stream: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    manager: Arc<RwLock<Option<SessionEntry>>>,
    session_id: String,
    mut shutdown_rx: mpsc::Receiver<()>,
) {
    let (mut ws_tx, mut ws_rx) = ws_stream.split();

    // Channel for outgoing WebSocket messages (from signaling forwarders)
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<String>();

    // Our own peer ID (set when we receive "connected")
    let mut local_peer_id: Option<String> = None;

    loop {
        tokio::select! {
            // Incoming message from PartyKit
            msg = ws_rx.next() => {
                let Some(msg) = msg else { break };
                let text = match msg {
                    Ok(tungstenite::Message::Text(t)) => t.to_string(),
                    Ok(tungstenite::Message::Close(_)) => break,
                    Err(e) => {
                        error!(session_id = %session_id, "WebSocket error: {e}");
                        break;
                    }
                    _ => continue,
                };

                handle_server_message(
                    &text,
                    &session_id,
                    &manager,
                    &out_tx,
                    &mut local_peer_id,
                ).await;
            }

            // Outgoing message to PartyKit
            Some(json) = out_rx.recv() => {
                if let Err(e) = ws_tx.send(tungstenite::Message::Text(json.into())).await {
                    error!(session_id = %session_id, "Failed to send WS message: {e}");
                    break;
                }
            }

            // Shutdown signal
            _ = shutdown_rx.recv() => {
                debug!(session_id = %session_id, "Signaling client shutting down");
                let _ = ws_tx.close().await;
                break;
            }
        }
    }

    info!(session_id = %session_id, "Signaling WebSocket loop ended");
}

/// Handle an incoming message from the PartyKit server.
async fn handle_server_message(
    text: &str,
    session_id: &str,
    manager: &Arc<RwLock<Option<SessionEntry>>>,
    out_tx: &mpsc::UnboundedSender<String>,
    local_peer_id: &mut Option<String>,
) {
    let value: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => {
            warn!(session_id = %session_id, "Invalid signaling message: {e}");
            return;
        }
    };

    let msg_type = value.get("type").and_then(|t| t.as_str()).unwrap_or("");

    match msg_type {
        "connected" => {
            let peer_id = value
                .get("peer_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            info!(session_id = %session_id, peer_id = %peer_id, "Connected as peer");
            *local_peer_id = Some(peer_id.to_string());
        }

        "room-peers" => {
            // Existing peers in the room when we join. In the p2p-stream flow,
            // the Rust worker joins first and waits for the browser, so this
            // list should be empty. If there are existing peers, treat them
            // like peer-joined.
            if let Some(peers) = value.get("peers").and_then(|p| p.as_array()) {
                for peer in peers {
                    if let Some(pid) = peer.as_str() {
                        create_peer_session(session_id, pid, manager, out_tx).await;
                    }
                }
            }
        }

        "peer-joined" => {
            let peer_id = value
                .get("peer_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            info!(session_id = %session_id, peer_id = %peer_id, "Browser peer joined");
            create_peer_session(session_id, peer_id, manager, out_tx).await;
        }

        "peer-left" => {
            let peer_id = value
                .get("peer_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            info!(session_id = %session_id, peer_id = %peer_id, "Browser peer left");

            let mut guard = manager.write().await;
            if let Some(entry) = guard.as_mut() {
                entry.active_peers.remove(peer_id);
                let _ = entry.manager.remove_session(peer_id);
            }
        }

        "offer" => {
            let from = value
                .get("from_peer_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let sdp = value
                .get("sdp")
                .and_then(|v| v.as_str())
                .unwrap_or_default();

            debug!(session_id = %session_id, from = %from, "Received SDP offer");

            let guard = manager.read().await;
            if let Some(entry) = guard.as_ref() {
                let msg = SignalingMessage::SessionDescription(SessionDescription {
                    sdp_type: SdpType::Offer,
                    sdp: sdp.to_string(),
                });
                if let Err(e) = entry.manager.handle_signaling_message(from, msg) {
                    error!(session_id = %session_id, "Failed to handle offer: {e}");
                }
            }
        }

        "answer" => {
            let from = value
                .get("from_peer_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let sdp = value
                .get("sdp")
                .and_then(|v| v.as_str())
                .unwrap_or_default();

            debug!(session_id = %session_id, from = %from, "Received SDP answer");

            let guard = manager.read().await;
            if let Some(entry) = guard.as_ref() {
                let msg = SignalingMessage::SessionDescription(SessionDescription {
                    sdp_type: SdpType::Answer,
                    sdp: sdp.to_string(),
                });
                if let Err(e) = entry.manager.handle_signaling_message(from, msg) {
                    error!(session_id = %session_id, "Failed to handle answer: {e}");
                }
            }
        }

        "ice-candidate" => {
            let from = value
                .get("from_peer_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let candidate_str = value
                .get("candidate")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let sdp_m_line_index = value
                .get("sdp_m_line_index")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;

            let guard = manager.read().await;
            if let Some(entry) = guard.as_ref() {
                let msg = SignalingMessage::IceCandidate(IceCandidate {
                    sdp_m_line_index,
                    candidate: candidate_str.to_string(),
                });
                if let Err(e) = entry.manager.handle_signaling_message(from, msg) {
                    error!(session_id = %session_id, "Failed to handle ICE candidate: {e}");
                }
            }
        }

        "error" => {
            let message = value
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            error!(session_id = %session_id, "Signaling error: {message}");
        }

        other => {
            debug!(session_id = %session_id, msg_type = %other, "Unknown signaling message type");
        }
    }
}

/// Create a peer session and start forwarding signaling messages.
async fn create_peer_session(
    session_id: &str,
    peer_id: &str,
    manager: &Arc<RwLock<Option<SessionEntry>>>,
    out_tx: &mpsc::UnboundedSender<String>,
) {
    let mut guard = manager.write().await;
    let Some(entry) = guard.as_mut() else { return };

    match entry.manager.create_session(peer_id) {
        Ok((_id, signaling_rx)) => {
            if let Err(e) = entry.manager.start_session(peer_id) {
                error!(
                    session_id = %session_id,
                    peer_id = %peer_id,
                    "Failed to start peer session: {e}"
                );
                let error_msg = serde_json::json!({
                    "type": "error",
                    "target_peer_id": peer_id,
                    "message": format!("Failed to start streaming session: {e}")
                });
                let _ = out_tx.send(error_msg.to_string());
                return;
            }

            entry.active_peers.insert(peer_id.to_string(), ());

            // Spawn a forwarder that reads from signaling_rx and translates
            // p2p-stream messages to PartyKit format, sending via out_tx.
            let out_tx = out_tx.clone();
            let peer_id_owned = peer_id.to_string();
            let session_id_owned = session_id.to_string();

            tokio::spawn(async move {
                forward_signaling_to_partykit(
                    signaling_rx,
                    &out_tx,
                    &peer_id_owned,
                    &session_id_owned,
                )
                .await;
            });

            info!(session_id = %session_id, peer_id = %peer_id, "Peer session created and started");
        }
        Err(e) => {
            error!(
                session_id = %session_id,
                peer_id = %peer_id,
                "Failed to create peer session: {e}"
            );
            let error_msg = serde_json::json!({
                "type": "error",
                "target_peer_id": peer_id,
                "message": format!("Failed to create streaming session: {e}")
            });
            let _ = out_tx.send(error_msg.to_string());
        }
    }
}

/// Forward outgoing signaling messages from p2p-stream to PartyKit format.
async fn forward_signaling_to_partykit(
    mut rx: mpsc::UnboundedReceiver<SignalingMessage>,
    out_tx: &mpsc::UnboundedSender<String>,
    target_peer_id: &str,
    session_id: &str,
) {
    while let Some(msg) = rx.recv().await {
        let partykit_msg = match &msg {
            SignalingMessage::SessionDescription(desc) => {
                let msg_type = match desc.sdp_type {
                    SdpType::Offer => "offer",
                    SdpType::Answer => "answer",
                };
                serde_json::json!({
                    "type": msg_type,
                    "target_peer_id": target_peer_id,
                    "sdp": desc.sdp,
                })
            }
            SignalingMessage::IceCandidate(ice) => {
                serde_json::json!({
                    "type": "ice-candidate",
                    "target_peer_id": target_peer_id,
                    "candidate": ice.candidate,
                    "sdp_m_line_index": ice.sdp_m_line_index,
                })
            }
            // Media control messages (Seek, MediaInfo, PositionUpdate) now flow
            // through the WebRTC data channel, not through signaling.
            _ => continue,
        };

        let json = partykit_msg.to_string();
        debug!(session_id = %session_id, "Sending to PartyKit: {json}");
        if out_tx.send(json).is_err() {
            break;
        }
    }

    debug!(session_id = %session_id, target_peer_id = %target_peer_id, "Signaling forwarder stopped");
}

// ===== Ethereum signing utilities =====

/// Derive an Ethereum address from a secp256k1 signing key.
fn eth_address_from_key(key: &SigningKey) -> String {
    let verifying_key = key.verifying_key();
    let encoded = verifying_key.to_encoded_point(false);
    // Skip the 0x04 prefix byte, hash the 64-byte uncompressed public key
    let bytes = &encoded.as_bytes()[1..];
    let hash = Keccak256::digest(bytes);
    // Ethereum address = last 20 bytes of keccak256
    format!("0x{}", hex_encode(&hash[12..]))
}

/// Build the WebSocket URL with Ethereum-signed auth parameters.
///
/// Matches the auth scheme in the PartyKit signaling server:
/// message = `partykit-auth:{roomId}:{timestamp}`
/// signature = EIP-191 personal sign of the message
fn build_ws_url(
    base_url: &str,
    room_id: &str,
    signing_key: &SigningKey,
    address: &str,
) -> Result<String, String> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();

    let message = format!("partykit-auth:{room_id}:{timestamp}");

    // EIP-191 personal sign: hash("\x19Ethereum Signed Message:\n{len}{message}")
    let prefixed = format!(
        "\x19Ethereum Signed Message:\n{}{}",
        message.len(),
        message
    );
    let hash = Keccak256::digest(prefixed.as_bytes());

    // Sign the hash with recovery
    let (signature, recovery_id) = signing_key
        .sign_prehash_recoverable(&hash)
        .map_err(|e| format!("Signing failed: {e}"))?;

    // Encode as 65-byte signature (r + s + v) where v = recovery_id + 27
    let mut sig_bytes = [0u8; 65];
    sig_bytes[..64].copy_from_slice(&signature.to_bytes());
    sig_bytes[64] = recovery_id.to_byte() + 27;

    let sig_hex = format!("0x{}", hex_encode(&sig_bytes));

    // Convert HTTP URL to WebSocket URL
    let ws_base = base_url
        .replace("https://", "wss://")
        .replace("http://", "ws://");

    Ok(format!(
        "{ws_base}/party/{room_id}?address={address}&signature={sig_hex}&timestamp={timestamp}"
    ))
}

/// Hex encode a byte slice.
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
