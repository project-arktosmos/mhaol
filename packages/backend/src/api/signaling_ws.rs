use crate::AppState;
use axum::{
    extract::{
        ws::{CloseFrame, Message, WebSocket},
        Path, Query, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc;

const AUTH_TIMESTAMP_MAX_AGE_MS: u64 = 30_000;

#[derive(Deserialize)]
pub struct AuthParams {
    address: String,
    signature: String,
    timestamp: String,
}

pub fn signaling_routes() -> Router<AppState> {
    Router::new()
        .route("/party/{room_id}", get(ws_handler))
        .route("/party/{room_id}/status", get(room_status))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    Query(params): Query<AuthParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Validate timestamp
    let ts: u64 = match params.timestamp.parse() {
        Ok(t) => t,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid timestamp").into_response(),
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    if now.abs_diff(ts) > AUTH_TIMESTAMP_MAX_AGE_MS {
        return (StatusCode::UNAUTHORIZED, "Expired timestamp").into_response();
    }

    // Verify EIP-191 signature
    let message = format!("partykit-auth:{}:{}", room_id, params.timestamp);
    let recovered =
        match crate::identity::passport::eip191_recover(&message, &params.signature) {
            Ok(addr) => addr,
            Err(_) => {
                return (StatusCode::UNAUTHORIZED, "Signature verification failed")
                    .into_response()
            }
        };

    if recovered.to_lowercase() != params.address.to_lowercase() {
        return (StatusCode::UNAUTHORIZED, "Signature mismatch").into_response();
    }

    let peer_id = recovered.to_lowercase();
    ws.on_upgrade(move |socket| handle_socket(socket, room_id, peer_id, state))
        .into_response()
}

async fn room_status(
    Path(room_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let peers = state.signaling_rooms.get_room_peers(&room_id);
    let count = peers.len();
    Json(serde_json::json!({
        "room_id": room_id,
        "peers": peers,
        "peerCount": count,
    }))
}

async fn handle_socket(socket: WebSocket, room_id: String, peer_id: String, state: AppState) {
    let (mut ws_tx, mut ws_rx) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    let connection_id = uuid::Uuid::new_v4().to_string();

    // Add peer to room, get existing peers and broadcast channels
    let (existing_peers, broadcast_txs) =
        state
            .signaling_rooms
            .add_peer(&room_id, &connection_id, &peer_id, tx);

    // Notify existing peers about the new peer
    let joined_msg = serde_json::json!({
        "type": "peer-joined",
        "room_id": room_id,
        "peer_id": peer_id,
    })
    .to_string();
    for btx in &broadcast_txs {
        let _ = btx.send(joined_msg.clone());
    }

    // Send connected + room-peers to the new client
    let connected_msg = serde_json::json!({
        "type": "connected",
        "peer_id": peer_id,
    })
    .to_string();
    let room_peers_msg = serde_json::json!({
        "type": "room-peers",
        "room_id": room_id,
        "peers": existing_peers,
    })
    .to_string();

    if ws_tx
        .send(Message::Text(connected_msg.into()))
        .await
        .is_err()
    {
        cleanup(&state, &room_id, &connection_id);
        return;
    }
    if ws_tx
        .send(Message::Text(room_peers_msg.into()))
        .await
        .is_err()
    {
        cleanup(&state, &room_id, &connection_id);
        return;
    }

    // Spawn task to forward outgoing messages from channel to WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if msg.starts_with("__close:") {
                let code: u16 = msg
                    .strip_prefix("__close:")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1000);
                let _ = ws_tx
                    .send(Message::Close(Some(CloseFrame {
                        code,
                        reason: "Replaced by new connection".into(),
                    })))
                    .await;
                break;
            }
            if ws_tx.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Receive incoming messages
    while let Some(result) = ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(_) => break,
        };

        match msg {
            Message::Text(text) => {
                handle_message(&state, &room_id, &peer_id, &text);
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    // Cleanup on disconnect
    cleanup(&state, &room_id, &connection_id);
    send_task.abort();
}

fn handle_message(state: &AppState, room_id: &str, from_peer_id: &str, text: &str) {
    let msg: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return,
    };

    let msg_type = match msg.get("type").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => return,
    };
    let target_peer_id = match msg.get("target_peer_id").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => return,
    };

    let relay_msg = match msg_type {
        "offer" => serde_json::json!({
            "type": "offer",
            "room_id": room_id,
            "from_peer_id": from_peer_id,
            "sdp": msg.get("sdp"),
        }),
        "answer" => serde_json::json!({
            "type": "answer",
            "room_id": room_id,
            "from_peer_id": from_peer_id,
            "sdp": msg.get("sdp"),
        }),
        "ice-candidate" => serde_json::json!({
            "type": "ice-candidate",
            "room_id": room_id,
            "from_peer_id": from_peer_id,
            "candidate": msg.get("candidate"),
            "sdp_m_line_index": msg.get("sdp_m_line_index"),
        }),
        _ => return,
    };

    state
        .signaling_rooms
        .relay_to_peer(room_id, target_peer_id, &relay_msg.to_string());
}

fn cleanup(state: &AppState, room_id: &str, connection_id: &str) {
    if let Some((peer_id, remaining_txs)) =
        state.signaling_rooms.remove_peer(room_id, connection_id)
    {
        let left_msg = serde_json::json!({
            "type": "peer-left",
            "room_id": room_id,
            "peer_id": peer_id,
        })
        .to_string();
        for tx in remaining_txs {
            let _ = tx.send(left_msg.clone());
        }
    }
}
