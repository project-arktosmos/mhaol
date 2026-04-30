use axum::{
    extract::{
        ws::{CloseFrame, Message, WebSocket},
        Path, Query, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::state::RendezvousState;
use crate::turn;

const AUTH_TIMESTAMP_MAX_AGE_MS: u64 = 30_000;

#[derive(Deserialize)]
pub struct AuthParams {
    address: String,
    signature: String,
    timestamp: String,
    passport_raw: Option<String>,
    passport_signature: Option<String>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    Query(params): Query<AuthParams>,
    State(state): State<RendezvousState>,
) -> impl IntoResponse {
    info!(room_id = %room_id, address = %params.address, "ws upgrade requested");

    let ts: u64 = match params.timestamp.parse() {
        Ok(t) => t,
        Err(_) => {
            warn!(room_id = %room_id, "ws auth rejected: invalid timestamp");
            return (StatusCode::UNAUTHORIZED, "Invalid timestamp").into_response();
        }
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    if now.abs_diff(ts) > AUTH_TIMESTAMP_MAX_AGE_MS {
        warn!(
            room_id = %room_id,
            client_ts = ts,
            server_ts = now,
            skew_ms = now.abs_diff(ts),
            "ws auth rejected: timestamp out of window"
        );
        return (StatusCode::UNAUTHORIZED, "Expired or invalid timestamp").into_response();
    }

    let message = format!("partykit-auth:{}:{}", room_id, params.timestamp);
    let recovered = match mhaol_identity::eip191_recover(&message, &params.signature) {
        Ok(addr) => addr,
        Err(e) => {
            warn!(room_id = %room_id, error = %e, "ws auth rejected: eip191 recover failed");
            return (StatusCode::UNAUTHORIZED, "Signature verification failed").into_response();
        }
    };

    if recovered.to_lowercase() != params.address.to_lowercase() {
        warn!(
            room_id = %room_id,
            recovered = %recovered,
            claimed = %params.address,
            "ws auth rejected: signature mismatch"
        );
        return (StatusCode::UNAUTHORIZED, "Signature mismatch").into_response();
    }

    let peer_id = recovered.to_lowercase();

    let (name, instance_type) = extract_passport_info(
        &peer_id,
        params.passport_raw.as_deref(),
        params.passport_signature.as_deref(),
    );

    info!(
        room_id = %room_id,
        peer_id = %peer_id,
        instance_type = %instance_type,
        "ws auth ok, upgrading"
    );

    ws.on_upgrade(move |socket| {
        handle_socket(socket, room_id, peer_id, name, instance_type, state)
    })
    .into_response()
}

fn extract_passport_info(
    address: &str,
    passport_raw: Option<&str>,
    passport_signature: Option<&str>,
) -> (String, String) {
    let (Some(raw), Some(sig)) = (passport_raw, passport_signature) else {
        return (String::new(), String::new());
    };

    let Ok(recovered) = mhaol_identity::eip191_recover(raw, sig) else {
        return (String::new(), String::new());
    };

    if recovered.to_lowercase() != address.to_lowercase() {
        return (String::new(), String::new());
    }

    let Ok(payload) = serde_json::from_str::<serde_json::Value>(raw) else {
        return (String::new(), String::new());
    };

    let name = payload
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let instance_type = payload
        .get("instanceType")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    (name, instance_type)
}

pub async fn room_status(
    Path(room_id): Path<String>,
    State(state): State<RendezvousState>,
) -> impl IntoResponse {
    let peers = state.rooms.get_room_peers(&room_id);
    Json(serde_json::json!({
        "room_id": room_id,
        "peers": peers.iter().map(|p| serde_json::json!({
            "peer_id": p.peer_id,
            "name": p.name,
            "instance_type": p.instance_type,
        })).collect::<Vec<_>>(),
        "peerCount": peers.len(),
    }))
}

async fn handle_socket(
    socket: WebSocket,
    room_id: String,
    peer_id: String,
    name: String,
    instance_type: String,
    state: RendezvousState,
) {
    let (mut ws_tx, mut ws_rx) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    let connection_id = uuid::Uuid::new_v4().to_string();

    let (existing_peers, broadcast_txs) =
        state
            .rooms
            .add_peer(&room_id, &connection_id, &peer_id, &name, &instance_type, tx);

    info!(
        room_id = %room_id,
        peer_id = %peer_id,
        existing_peers = existing_peers.len(),
        "peer joined"
    );

    let joined_msg = serde_json::json!({
        "type": "peer-joined",
        "room_id": room_id,
        "peer_id": peer_id,
        "name": name,
        "instance_type": instance_type,
    })
    .to_string();
    for btx in &broadcast_txs {
        let _ = btx.send(joined_msg.clone());
    }

    let ice_servers = turn::generate_credentials(&state.turn);

    let connected_msg = serde_json::json!({
        "type": "connected",
        "peer_id": peer_id,
        "name": name,
        "instance_type": instance_type,
        "ice_servers": ice_servers,
    })
    .to_string();

    let room_peers_msg = serde_json::json!({
        "type": "room-peers",
        "room_id": room_id,
        "peers": existing_peers.iter().map(|p| serde_json::json!({
            "peer_id": p.peer_id,
            "name": p.name,
            "instance_type": p.instance_type,
        })).collect::<Vec<_>>(),
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

    cleanup(&state, &room_id, &connection_id);
    send_task.abort();
}

fn handle_message(state: &RendezvousState, room_id: &str, from_peer_id: &str, text: &str) {
    let msg: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(e) => {
            warn!(room_id = %room_id, from = %from_peer_id, error = %e, "ws message: invalid json");
            return;
        }
    };

    let msg_type = match msg.get("type").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => {
            warn!(room_id = %room_id, from = %from_peer_id, "ws message: missing 'type'");
            return;
        }
    };
    let target_peer_id = match msg.get("target_peer_id").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => {
            warn!(
                room_id = %room_id,
                from = %from_peer_id,
                msg_type = %msg_type,
                "ws message: missing 'target_peer_id'"
            );
            return;
        }
    };

    debug!(
        room_id = %room_id,
        from = %from_peer_id,
        target = %target_peer_id,
        msg_type = %msg_type,
        "relaying"
    );

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
            "sdp_mid": msg.get("sdp_mid"),
        }),
        _ => return,
    };

    state
        .rooms
        .relay_to_peer(room_id, target_peer_id, &relay_msg.to_string());
}

fn cleanup(state: &RendezvousState, room_id: &str, connection_id: &str) {
    if let Some((peer_info, remaining_txs)) = state.rooms.remove_peer(room_id, connection_id) {
        info!(
            room_id = %room_id,
            peer_id = %peer_info.peer_id,
            remaining = remaining_txs.len(),
            "peer left"
        );
        let left_msg = serde_json::json!({
            "type": "peer-left",
            "room_id": room_id,
            "peer_id": peer_info.peer_id,
        })
        .to_string();
        for tx in remaining_txs {
            let _ = tx.send(left_msg.clone());
        }
    }
}
