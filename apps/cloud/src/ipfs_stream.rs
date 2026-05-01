//! `/api/ipfs-stream` — start an HLS transcoding session for a previously
//! pinned IPFS file and serve the resulting playlist + segments over HTTP.
//!
//! The cloud already has the file on disk via `ipfs_pin` (segments are
//! pulled into the IPFS repo when libraries are scanned or torrents finish).
//! This route reuses that local copy as the input to a GStreamer pipeline
//! that emits a VOD-style HLS playlist; the browser then plays it via
//! `hls.js` (or native HLS on Safari).
//!
//! Conceptually the transport is "HLS over IPFS": the segments originate
//! from the IPFS swarm before being transcoded, and the playlist is served
//! from the cloud the swarm is bootstrapped against.

use crate::state::CloudState;
use axum::{
    body::Body,
    extract::{Path as AxumPath, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tokio_util::io::ReaderStream;

#[cfg(not(target_os = "android"))]
use crate::ipfs_pins::{IpfsPin, TABLE as PIN_TABLE};

#[derive(Debug, Deserialize)]
pub struct StartSessionRequest {
    pub cid: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartSessionResponse {
    pub session_id: String,
    pub playlist_url: String,
    pub playlist_ready: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionDto {
    pub session_id: String,
    pub cid: String,
    pub state: String,
    pub playlist_url: String,
}

pub fn router() -> Router<CloudState> {
    Router::new()
        .route("/sessions", post(start_session).get(list_sessions))
        .route("/sessions/{id}", delete(stop_session))
        // Both the playlist and the segments live in the same on-disk dir
        // and the m3u8 references segments by relative filename. Serving
        // them at the same path level lets `hls.js` resolve segment URLs
        // correctly against the playlist URL without rewriting the m3u8.
        .route("/sessions/{id}/playlist.m3u8", get(serve_playlist))
        .route("/sessions/{id}/{filename}", get(serve_segment))
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

    if let Err(e) = mhaol_ipfs_stream::init() {
        return Err(err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("ipfs-stream init failed: {e}"),
        ));
    }

    let manager = state.ipfs_stream_manager.clone();
    let manager_for_wait = manager.clone();
    let started = manager
        .start_session(cid.clone(), PathBuf::from(&pin.path))
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, format!("start_session: {e}")))?;
    let session_id = started.session_id.clone();

    // Block briefly so the first segment lands before the player starts
    // requesting the playlist. hlssink2 only flushes the m3u8 once a
    // segment is closed; without this wait the player would 404 the first
    // request and bail.
    let session_for_wait = session_id.clone();
    let ready = tokio::task::spawn_blocking(move || {
        manager_for_wait.wait_for_playlist(&session_for_wait, Duration::from_secs(20))
    })
    .await
    .unwrap_or(false);

    // Probe the source's full duration so the player can render a real
    // total even though the rolling HLS playlist still looks live.
    // decodebin reports duration shortly after the pipeline reaches
    // PLAYING, which `wait_for_playlist` already implies.
    let manager_for_dur = manager.clone();
    let session_for_dur = session_id.clone();
    let duration_seconds = tokio::task::spawn_blocking(move || {
        manager_for_dur.query_source_duration(&session_for_dur, Duration::from_secs(5))
    })
    .await
    .ok()
    .flatten();

    tracing::info!(
        "[ipfs-stream] session {session_id} cid={cid} ready={ready} duration={:?}s",
        duration_seconds
    );

    Ok(Json(StartSessionResponse {
        session_id: session_id.clone(),
        playlist_url: format!("/api/ipfs-stream/sessions/{session_id}/playlist.m3u8"),
        playlist_ready: ready,
        duration_seconds,
    }))
}

#[cfg(target_os = "android")]
async fn start_session(
    State(_state): State<CloudState>,
    Json(_req): Json<StartSessionRequest>,
) -> Result<Json<StartSessionResponse>, (StatusCode, Json<serde_json::Value>)> {
    Err(err(
        StatusCode::SERVICE_UNAVAILABLE,
        "ipfs-stream not available on this platform",
    ))
}

#[cfg(not(target_os = "android"))]
async fn list_sessions(
    State(state): State<CloudState>,
) -> Json<Vec<SessionDto>> {
    let sessions = state.ipfs_stream_manager.list_sessions();
    let dtos = sessions
        .into_iter()
        .map(|s| SessionDto {
            session_id: s.session_id.clone(),
            cid: s.cid,
            state: format!("{:?}", s.state).to_lowercase(),
            playlist_url: format!("/api/ipfs-stream/sessions/{}/playlist.m3u8", s.session_id),
        })
        .collect();
    Json(dtos)
}

#[cfg(target_os = "android")]
async fn list_sessions(State(_state): State<CloudState>) -> Json<Vec<SessionDto>> {
    Json(Vec::new())
}

#[cfg(not(target_os = "android"))]
async fn stop_session(
    State(state): State<CloudState>,
    AxumPath(id): AxumPath<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    state
        .ipfs_stream_manager
        .stop_session(&id)
        .map_err(|e| err(StatusCode::NOT_FOUND, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(target_os = "android")]
async fn stop_session(
    State(_state): State<CloudState>,
    AxumPath(_id): AxumPath<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    Err(err(
        StatusCode::SERVICE_UNAVAILABLE,
        "ipfs-stream not available on this platform",
    ))
}

#[cfg(not(target_os = "android"))]
async fn serve_playlist(
    State(state): State<CloudState>,
    AxumPath(id): AxumPath<String>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let info = state
        .ipfs_stream_manager
        .get_session(&id)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "session not found"))?;
    let playlist_path = PathBuf::from(&info.output_dir).join(&info.playlist_name);
    serve_file(playlist_path, "application/vnd.apple.mpegurl").await
}

#[cfg(target_os = "android")]
async fn serve_playlist(
    State(_state): State<CloudState>,
    AxumPath(_id): AxumPath<String>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    Err(err(StatusCode::SERVICE_UNAVAILABLE, "not available"))
}

#[cfg(not(target_os = "android"))]
async fn serve_segment(
    State(state): State<CloudState>,
    AxumPath((id, filename)): AxumPath<(String, String)>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err(err(StatusCode::BAD_REQUEST, "invalid segment name"));
    }
    let info = state
        .ipfs_stream_manager
        .get_session(&id)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "session not found"))?;
    let segment_path = PathBuf::from(&info.output_dir).join(&filename);
    serve_file(segment_path, "video/mp2t").await
}

#[cfg(target_os = "android")]
async fn serve_segment(
    State(_state): State<CloudState>,
    AxumPath((_id, _filename)): AxumPath<(String, String)>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    Err(err(StatusCode::SERVICE_UNAVAILABLE, "not available"))
}

#[cfg(not(target_os = "android"))]
async fn serve_file(
    path: PathBuf,
    content_type: &'static str,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let file = fs::File::open(&path)
        .await
        .map_err(|_| err(StatusCode::NOT_FOUND, format!("file not found: {}", path.display())))?;
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            (header::CACHE_CONTROL, "no-store"),
        ],
        body,
    )
        .into_response())
}
