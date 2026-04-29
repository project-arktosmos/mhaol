//! `/api/p2p-stream` — start a WebRTC streaming session for a previously
//! pinned IPFS file. The cloud spawns a `mhaol-cloud worker` subprocess
//! (see `worker_bridge.rs`), pushes a session into PartyKit, and returns
//! `{ sessionId, roomId, signalingUrl }` so the player can connect to the
//! same room and consume the stream over WebRTC.

use crate::state::CloudState;
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[cfg(not(target_os = "android"))]
use crate::ipfs_pins::{IpfsPin, TABLE as PIN_TABLE};
#[cfg(not(target_os = "android"))]
use crate::worker_bridge::WorkerEvent;

#[derive(Debug, Deserialize)]
pub struct StartSessionRequest {
    pub cid: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartSessionResponse {
    pub session_id: String,
    pub room_id: String,
    pub signaling_url: String,
}

pub fn router() -> Router<CloudState> {
    Router::new().route("/sessions", post(start_session))
}

fn err(status: StatusCode, message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(json!({ "error": message.into() })))
}

#[cfg(not(target_os = "android"))]
async fn start_session(
    State(state): State<CloudState>,
    Json(req): Json<StartSessionRequest>,
) -> Result<Json<StartSessionResponse>, (StatusCode, Json<serde_json::Value>)> {
    let cid = req.cid.trim().to_string();
    if cid.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "cid is required"));
    }

    if !state.worker_bridge.is_ready() {
        return Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            "streaming worker is not running",
        ));
    }

    let pins: Vec<IpfsPin> = state
        .db
        .select(PIN_TABLE)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    let pin = pins
        .into_iter()
        .find(|p| p.cid == cid)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, format!("no local file for cid {cid}")))?;

    if !std::path::Path::new(&pin.path).exists() {
        return Err(err(
            StatusCode::NOT_FOUND,
            format!("file no longer on disk: {}", pin.path),
        ));
    }

    let session_id = uuid::Uuid::new_v4().to_string();
    let event = state
        .worker_bridge
        .create_session(
            &session_id,
            &pin.path,
            None,
            &state.signaling_url,
            Some("video".to_string()),
            None,
            None,
            None,
        )
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e))?;

    match event {
        WorkerEvent::SessionCreated {
            session_id,
            room_id,
        } => Ok(Json(StartSessionResponse {
            session_id,
            room_id,
            signaling_url: state.signaling_url.clone(),
        })),
        WorkerEvent::Error { error, .. } => Err(err(StatusCode::INTERNAL_SERVER_ERROR, error)),
        WorkerEvent::SessionDeleted { .. } => Err(err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "unexpected session_deleted from worker",
        )),
    }
}

#[cfg(target_os = "android")]
async fn start_session(
    State(_state): State<CloudState>,
    Json(_req): Json<StartSessionRequest>,
) -> Result<Json<StartSessionResponse>, (StatusCode, Json<serde_json::Value>)> {
    Err(err(
        StatusCode::SERVICE_UNAVAILABLE,
        "p2p-stream not available on this platform",
    ))
}
