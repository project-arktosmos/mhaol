use crate::state::CloudState;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

#[cfg(not(target_os = "android"))]
use mhaol_torrent::{AddTorrentRequest, TorrentInfo};

#[derive(Debug, Deserialize)]
pub struct AddRequest {
    pub magnet: String,
}

pub fn router() -> Router<CloudState> {
    Router::new()
        .route("/list", get(list))
        .route("/add", post(add))
}

fn err(status: StatusCode, message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(json!({ "error": message.into() })))
}

#[cfg(not(target_os = "android"))]
async fn list(
    State(state): State<CloudState>,
) -> Result<Json<Vec<TorrentInfo>>, (StatusCode, Json<serde_json::Value>)> {
    if !state.torrent_manager.is_initialized() {
        return Ok(Json(Vec::new()));
    }
    state
        .torrent_manager
        .list()
        .await
        .map(Json)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[cfg(target_os = "android")]
async fn list(
    State(_state): State<CloudState>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(Vec::new()))
}

#[cfg(not(target_os = "android"))]
async fn add(
    State(state): State<CloudState>,
    Json(req): Json<AddRequest>,
) -> Result<Json<TorrentInfo>, (StatusCode, Json<serde_json::Value>)> {
    let magnet = req.magnet.trim();
    if !magnet.starts_with("magnet:") {
        return Err(err(StatusCode::BAD_REQUEST, "magnet URI required"));
    }
    if !state.torrent_manager.is_initialized() {
        return Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            "torrent client not ready",
        ));
    }
    state
        .torrent_manager
        .add(AddTorrentRequest {
            source: magnet.to_string(),
            download_path: None,
            paused: None,
        })
        .await
        .map(Json)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

#[cfg(target_os = "android")]
async fn add(
    State(_state): State<CloudState>,
    Json(_req): Json<AddRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    Err(err(
        StatusCode::SERVICE_UNAVAILABLE,
        "torrent client unavailable on this platform",
    ))
}
