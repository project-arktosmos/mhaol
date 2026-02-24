use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::{IntoResponse, Json},
};
use futures_util::{SinkExt, StreamExt};
use mhaol_p2p_stream::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

// ===== Shared State =====

pub struct SessionEntry {
    pub manager: SessionManager,
    pub file_path: String,
    pub created_at: Instant,
}

pub struct AppState {
    pub sessions: RwLock<HashMap<String, SessionEntry>>,
}

// ===== Request/Response Types =====

#[derive(Deserialize)]
pub struct CreateSessionRequest {
    pub file_path: String,
    pub mode: Option<String>,
}

#[derive(Serialize)]
pub struct CreateSessionResponse {
    pub session_id: String,
    pub ws_url: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ===== Health Check =====

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

// ===== Create Session =====

pub async fn create_session(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<CreateSessionResponse>, (StatusCode, Json<ErrorResponse>)> {
    let path = PathBuf::from(&req.file_path);
    if !path.exists() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("File not found: {}", req.file_path),
            }),
        ));
    }

    let is_audio_only = req.mode.as_deref() == Some("audio");

    let file_source = if is_audio_only {
        FileSource::new(&path).audio_only()
    } else {
        FileSource::new(&path)
    };

    let manager = SessionManager::new(|| PipelineBuilder::new(), file_source);

    let session_id = Uuid::new_v4().to_string();

    info!(
        session_id = %session_id,
        file_path = %req.file_path,
        mode = ?req.mode,
        "Created streaming session"
    );

    state.sessions.write().await.insert(
        session_id.clone(),
        SessionEntry {
            manager,
            file_path: req.file_path,
            created_at: Instant::now(),
        },
    );

    Ok(Json(CreateSessionResponse {
        ws_url: format!("/sessions/{}/ws", session_id),
        session_id,
    }))
}

// ===== WebSocket Signaling =====

pub async fn ws_handler(
    Path(session_id): Path<String>,
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, session_id, state))
}

async fn handle_ws(socket: WebSocket, session_id: String, state: Arc<AppState>) {
    let peer_id = Uuid::new_v4().to_string();

    info!(
        session_id = %session_id,
        peer_id = %peer_id,
        "WebSocket connected"
    );

    // Look up the session and create a peer
    let signaling_rx = {
        let sessions = state.sessions.read().await;
        let Some(entry) = sessions.get(&session_id) else {
            warn!(session_id = %session_id, "Session not found for WebSocket");
            return;
        };

        match entry.manager.create_session(&peer_id) {
            Ok((_id, rx)) => rx,
            Err(e) => {
                error!(
                    session_id = %session_id,
                    peer_id = %peer_id,
                    error = %e,
                    "Failed to create peer session"
                );
                return;
            }
        }
    };

    // Start the session (begins pipeline playback and triggers negotiation)
    {
        let sessions = state.sessions.read().await;
        if let Some(entry) = sessions.get(&session_id) {
            if let Err(e) = entry.manager.start_session(&peer_id) {
                error!(
                    session_id = %session_id,
                    peer_id = %peer_id,
                    error = %e,
                    "Failed to start peer session"
                );
                return;
            }
        }
    }

    let (ws_sender, ws_receiver) = socket.split();

    // Spawn outgoing signaling: p2p-stream -> WebSocket -> browser
    let outgoing_handle = tokio::spawn(forward_outgoing(signaling_rx, ws_sender));

    // Spawn incoming signaling: browser -> WebSocket -> p2p-stream
    let incoming_handle = tokio::spawn(forward_incoming(
        ws_receiver,
        session_id.clone(),
        peer_id.clone(),
        state.clone(),
    ));

    // Wait for either task to finish
    tokio::select! {
        _ = outgoing_handle => {},
        _ = incoming_handle => {},
    }

    // Clean up the peer session
    {
        let sessions = state.sessions.read().await;
        if let Some(entry) = sessions.get(&session_id) {
            let _ = entry.manager.remove_session(&peer_id);
        }
    }

    info!(
        session_id = %session_id,
        peer_id = %peer_id,
        "WebSocket disconnected"
    );
}

async fn forward_outgoing(
    mut signaling_rx: tokio::sync::mpsc::UnboundedReceiver<SignalingMessage>,
    mut ws_sender: futures_util::stream::SplitSink<WebSocket, Message>,
) {
    while let Some(msg) = signaling_rx.recv().await {
        match serde_json::to_string(&msg) {
            Ok(json) => {
                if ws_sender.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
            Err(e) => {
                error!(error = %e, "Failed to serialize signaling message");
            }
        }
    }
}

async fn forward_incoming(
    mut ws_receiver: futures_util::stream::SplitStream<WebSocket>,
    session_id: String,
    peer_id: String,
    state: Arc<AppState>,
) {
    while let Some(Ok(msg)) = ws_receiver.next().await {
        let text = match msg {
            Message::Text(t) => t,
            Message::Close(_) => break,
            _ => continue,
        };

        let signaling_msg: SignalingMessage = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(e) => {
                warn!(error = %e, "Failed to parse signaling message from browser");
                continue;
            }
        };

        let sessions = state.sessions.read().await;
        if let Some(entry) = sessions.get(&session_id) {
            if let Err(e) = entry.manager.handle_signaling_message(&peer_id, signaling_msg) {
                error!(
                    session_id = %session_id,
                    peer_id = %peer_id,
                    error = %e,
                    "Failed to handle signaling message"
                );
            }
        } else {
            break;
        }
    }
}

// ===== Delete Session =====

pub async fn delete_session(
    Path(session_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> StatusCode {
    if state.sessions.write().await.remove(&session_id).is_some() {
        info!(session_id = %session_id, "Session deleted");
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}
