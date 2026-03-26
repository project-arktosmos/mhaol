use super::types::{parse_signaling_message, SignalingServerMessage};
use futures_util::{SinkExt, StreamExt};
use k256::ecdsa::SigningKey;
use sha3::{Digest, Keccak256};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite;
use tracing::{debug, error, info};

/// A connected PartyKit room client.
///
/// Manages a WebSocket connection to a single PartyKit room, parsing incoming
/// messages into typed `SignalingServerMessage` and accepting outgoing JSON
/// strings to send.
pub struct PartyKitClient {
    room_id: String,
    /// Send outgoing JSON messages to the WebSocket.
    pub out_tx: mpsc::UnboundedSender<String>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl PartyKitClient {
    /// Connect to a PartyKit signaling room.
    ///
    /// Returns the client handle and a receiver for incoming server messages.
    /// The WebSocket loop runs as a background tokio task.
    pub async fn connect(
        signaling_url: &str,
        room_id: &str,
        private_key_hex: &str,
        passport_raw: &str,
        passport_signature: &str,
        endorser_signature: Option<&str>,
    ) -> Result<(Self, mpsc::UnboundedReceiver<SignalingServerMessage>), String> {
        let ws_url = build_ws_url(
            signaling_url,
            room_id,
            private_key_hex,
            passport_raw,
            passport_signature,
            endorser_signature,
        )?;

        info!(room_id = %room_id, "Connecting to PartyKit room");

        let (ws_stream, _response) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .map_err(|e| format!("WebSocket connect failed: {e}"))?;

        info!(room_id = %room_id, "Connected to PartyKit room");

        let (msg_tx, msg_rx) = mpsc::unbounded_channel();
        let (out_tx, out_rx) = mpsc::unbounded_channel();
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);

        let room_id_owned = room_id.to_string();
        tokio::spawn(async move {
            run_ws_loop(ws_stream, msg_tx, out_rx, room_id_owned, shutdown_rx).await;
        });

        Ok((
            Self {
                room_id: room_id.to_string(),
                out_tx,
                shutdown_tx: Some(shutdown_tx),
            },
            msg_rx,
        ))
    }

    /// Send a JSON message to the PartyKit room.
    pub fn send(&self, json: String) -> Result<(), String> {
        self.out_tx
            .send(json)
            .map_err(|e| format!("Failed to send: {e}"))
    }

    /// Send an SDP offer to a specific peer.
    pub fn send_offer(&self, target_peer_id: &str, sdp: &str) -> Result<(), String> {
        let msg = serde_json::json!({
            "type": "offer",
            "target_peer_id": target_peer_id,
            "sdp": sdp,
        });
        self.send(msg.to_string())
    }

    /// Send an SDP answer to a specific peer.
    pub fn send_answer(&self, target_peer_id: &str, sdp: &str) -> Result<(), String> {
        let msg = serde_json::json!({
            "type": "answer",
            "target_peer_id": target_peer_id,
            "sdp": sdp,
        });
        self.send(msg.to_string())
    }

    /// Send an ICE candidate to a specific peer.
    pub fn send_ice_candidate(
        &self,
        target_peer_id: &str,
        candidate: &str,
        sdp_m_line_index: u32,
        sdp_mid: Option<&str>,
    ) -> Result<(), String> {
        let mut msg = serde_json::json!({
            "type": "ice-candidate",
            "target_peer_id": target_peer_id,
            "candidate": candidate,
            "sdp_m_line_index": sdp_m_line_index,
        });
        if let Some(mid) = sdp_mid {
            msg["sdp_mid"] = serde_json::Value::String(mid.to_string());
        }
        self.send(msg.to_string())
    }

    pub fn room_id(&self) -> &str {
        &self.room_id
    }

    /// Disconnect from the room.
    pub fn disconnect(&mut self) {
        self.shutdown_tx.take();
    }
}

impl Drop for PartyKitClient {
    fn drop(&mut self) {
        self.disconnect();
    }
}

/// Main WebSocket event loop for a single room connection.
async fn run_ws_loop(
    ws_stream: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    msg_tx: mpsc::UnboundedSender<SignalingServerMessage>,
    mut out_rx: mpsc::UnboundedReceiver<String>,
    room_id: String,
    mut shutdown_rx: mpsc::Receiver<()>,
) {
    let (mut ws_tx, mut ws_rx) = ws_stream.split();

    loop {
        tokio::select! {
            msg = ws_rx.next() => {
                let Some(msg) = msg else { break };
                let text = match msg {
                    Ok(tungstenite::Message::Text(t)) => t.to_string(),
                    Ok(tungstenite::Message::Close(_)) => break,
                    Err(e) => {
                        error!(room_id = %room_id, "WebSocket error: {e}");
                        break;
                    }
                    _ => continue,
                };

                if let Some(parsed) = parse_signaling_message(&text) {
                    if msg_tx.send(parsed).is_err() {
                        break;
                    }
                } else {
                    debug!(room_id = %room_id, "Unknown signaling message: {text}");
                }
            }

            Some(json) = out_rx.recv() => {
                if let Err(e) = ws_tx.send(tungstenite::Message::Text(json.into())).await {
                    error!(room_id = %room_id, "Failed to send WS message: {e}");
                    break;
                }
            }

            _ = shutdown_rx.recv() => {
                debug!(room_id = %room_id, "PartyKit client shutting down");
                let _ = ws_tx.close().await;
                break;
            }
        }
    }

    info!(room_id = %room_id, "PartyKit WebSocket loop ended");
}

/// Build the WebSocket URL with EIP-191 signed auth parameters.
fn build_ws_url(
    base_url: &str,
    room_id: &str,
    private_key_hex: &str,
    passport_raw: &str,
    passport_signature: &str,
    endorser_signature: Option<&str>,
) -> Result<String, String> {
    let hex_str = private_key_hex
        .strip_prefix("0x")
        .unwrap_or(private_key_hex);
    let key_bytes = hex::decode(hex_str).map_err(|e| format!("Invalid key hex: {e}"))?;
    let signing_key = SigningKey::from_bytes((&key_bytes[..]).into())
        .map_err(|e| format!("Invalid signing key: {e}"))?;

    let address = eth_address_from_key(&signing_key);

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();

    let message = format!("partykit-auth:{room_id}:{timestamp}");

    let prefixed = format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message);
    let hash = Keccak256::digest(prefixed.as_bytes());

    let (signature, recovery_id) = signing_key
        .sign_prehash_recoverable(&hash)
        .map_err(|e| format!("Signing failed: {e}"))?;

    let mut sig_bytes = [0u8; 65];
    sig_bytes[..64].copy_from_slice(&signature.to_bytes());
    sig_bytes[64] = recovery_id.to_byte() + 27;

    let sig_hex = format!("0x{}", hex_encode(&sig_bytes));

    let ws_base = base_url
        .replace("https://", "wss://")
        .replace("http://", "ws://");

    let mut url = format!(
        "{ws_base}/party/{room_id}?address={address}&signature={sig_hex}&timestamp={timestamp}&passport_raw={}&passport_signature={}",
        percent_encode(passport_raw),
        percent_encode(passport_signature),
    );

    if let Some(endorser_sig) = endorser_signature {
        url.push_str(&format!(
            "&endorser_signature={}",
            percent_encode(endorser_sig),
        ));
    }

    Ok(url)
}

fn eth_address_from_key(key: &SigningKey) -> String {
    let verifying_key = key.verifying_key();
    let encoded = verifying_key.to_encoded_point(false);
    let bytes = &encoded.as_bytes()[1..];
    let hash = Keccak256::digest(bytes);
    format!("0x{}", hex_encode(&hash[12..]))
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn percent_encode(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 3);
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{byte:02X}"));
            }
        }
    }
    result
}
