use crate::state::CloudState;
use axum::{
    extract::{Path as AxumPath, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

#[cfg(target_os = "android")]
use axum::response::IntoResponse;
#[cfg(not(target_os = "android"))]
use serde::Serialize;

#[cfg(not(target_os = "android"))]
use axum::{
    body::Body,
    http::{header, HeaderMap, HeaderValue, Response},
};
#[cfg(not(target_os = "android"))]
use mhaol_torrent::{AddTorrentRequest, TorrentInfo, TorrentStreamInfo};
#[cfg(not(target_os = "android"))]
use tokio::io::{AsyncReadExt, AsyncSeekExt};
#[cfg(not(target_os = "android"))]
use tokio_util::io::ReaderStream;

#[derive(Debug, Deserialize)]
pub struct AddRequest {
    pub magnet: String,
}

#[derive(Debug, Deserialize)]
pub struct StreamRequest {
    pub magnet: String,
}

#[cfg(not(target_os = "android"))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StreamStartResponse {
    info_hash: String,
    name: String,
    file_index: usize,
    file_name: String,
    file_size: u64,
    mime_type: Option<String>,
    stream_url: String,
}

#[cfg(not(target_os = "android"))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EvaluateOk {
    streamable: bool,
    info_hash: String,
    name: String,
    file_index: usize,
    file_name: String,
    file_size: u64,
    mime_type: Option<String>,
}

#[cfg(not(target_os = "android"))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EvaluateNotStreamable {
    streamable: bool,
    reason: String,
}

pub fn router() -> Router<CloudState> {
    Router::new()
        .route("/list", get(list))
        .route("/add", post(add))
        .route("/stream", post(stream_start))
        .route("/stream/{info_hash}/{file_index}", get(stream_serve))
        .route("/evaluate", post(evaluate))
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

#[cfg(not(target_os = "android"))]
async fn stream_start(
    State(state): State<CloudState>,
    Json(req): Json<StreamRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
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

    let TorrentStreamInfo {
        info_hash,
        name,
        file,
    } = state
        .torrent_manager
        .start_stream(magnet)
        .await
        .map_err(|e| err(StatusCode::BAD_REQUEST, e.to_string()))?;

    let stream_url = format!("/api/torrent/stream/{}/{}", info_hash, file.index);
    let body = StreamStartResponse {
        info_hash,
        name,
        file_index: file.index,
        file_name: file.name,
        file_size: file.size,
        mime_type: file.mime_type,
        stream_url,
    };
    Ok(Json(serde_json::to_value(body).unwrap_or_default()))
}

#[cfg(target_os = "android")]
async fn stream_start(
    State(_state): State<CloudState>,
    Json(_req): Json<StreamRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    Err(err(
        StatusCode::SERVICE_UNAVAILABLE,
        "torrent streaming unavailable on this platform",
    ))
}

/// Probe a magnet without committing to a download. Resolves metadata via
/// librqbit's `list_only` flag (DHT + tracker peer discovery + BEP 9/10
/// metadata exchange — no piece downloads, no on-disk side-effects) and
/// reports whether the torrent has a streamable video file. Used by the
/// WebUI to enable / disable the "Torrent Stream" button up-front.
///
/// Always returns 200; the JSON `streamable` field is the discriminator.
/// Failures (no peers, malformed magnet, metadata timeout) become
/// `{ streamable: false, reason: "…" }` rather than HTTP errors so the UI
/// can keep the button disabled without showing an error toast.
#[cfg(not(target_os = "android"))]
async fn evaluate(
    State(state): State<CloudState>,
    Json(req): Json<StreamRequest>,
) -> Json<serde_json::Value> {
    let magnet = req.magnet.trim();
    if !magnet.starts_with("magnet:") {
        return Json(
            serde_json::to_value(EvaluateNotStreamable {
                streamable: false,
                reason: "magnet URI required".into(),
            })
            .unwrap_or_default(),
        );
    }
    if !state.torrent_manager.is_initialized() {
        return Json(
            serde_json::to_value(EvaluateNotStreamable {
                streamable: false,
                reason: "torrent client not ready".into(),
            })
            .unwrap_or_default(),
        );
    }

    match state.torrent_manager.evaluate_magnet(magnet).await {
        Ok(TorrentStreamInfo {
            info_hash,
            name,
            file,
        }) => Json(
            serde_json::to_value(EvaluateOk {
                streamable: true,
                info_hash,
                name,
                file_index: file.index,
                file_name: file.name,
                file_size: file.size,
                mime_type: file.mime_type,
            })
            .unwrap_or_default(),
        ),
        Err(e) => Json(
            serde_json::to_value(EvaluateNotStreamable {
                streamable: false,
                reason: e.to_string(),
            })
            .unwrap_or_default(),
        ),
    }
}

#[cfg(target_os = "android")]
async fn evaluate(
    State(_state): State<CloudState>,
    Json(_req): Json<StreamRequest>,
) -> Json<serde_json::Value> {
    Json(json!({
        "streamable": false,
        "reason": "torrent streaming unavailable on this platform"
    }))
}

#[cfg(not(target_os = "android"))]
fn parse_range(value: &HeaderValue, file_len: u64) -> Option<(u64, u64)> {
    let s = value.to_str().ok()?;
    let s = s.strip_prefix("bytes=")?;
    // Only honor the first range; tail-range "bytes=-N" is supported.
    let first = s.split(',').next()?.trim();
    let (start_s, end_s) = first.split_once('-')?;
    if start_s.is_empty() {
        let suffix: u64 = end_s.parse().ok()?;
        if suffix == 0 || file_len == 0 {
            return None;
        }
        let suffix = suffix.min(file_len);
        return Some((file_len - suffix, file_len - 1));
    }
    let start: u64 = start_s.parse().ok()?;
    let end: u64 = if end_s.is_empty() {
        file_len.saturating_sub(1)
    } else {
        end_s.parse().ok()?
    };
    if start > end || start >= file_len {
        return None;
    }
    Some((start, end.min(file_len.saturating_sub(1))))
}

#[cfg(not(target_os = "android"))]
async fn stream_serve(
    State(state): State<CloudState>,
    AxumPath((info_hash, file_index)): AxumPath<(String, usize)>,
    headers: HeaderMap,
) -> Result<Response<Body>, (StatusCode, Json<serde_json::Value>)> {
    if !state.torrent_manager.is_initialized() {
        return Err(err(
            StatusCode::SERVICE_UNAVAILABLE,
            "torrent client not ready",
        ));
    }

    let (file_len, mut stream) = state
        .torrent_manager
        .open_file_stream(&info_hash, file_index)
        .map_err(|e| err(StatusCode::NOT_FOUND, e.to_string()))?;

    let mime = headers
        .get("x-mhaol-mime")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());

    let range_header = headers.get(header::RANGE).cloned();
    if let Some(value) = range_header {
        let (start, end) = parse_range(&value, file_len).ok_or_else(|| {
            (
                StatusCode::RANGE_NOT_SATISFIABLE,
                Json(json!({ "error": "invalid Range header" })),
            )
        })?;
        let length = end - start + 1;
        stream
            .seek(std::io::SeekFrom::Start(start))
            .await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, format!("seek: {e}")))?;
        let limited = stream.take(length);
        let body = Body::from_stream(ReaderStream::new(limited));
        let mut resp = Response::builder()
            .status(StatusCode::PARTIAL_CONTENT)
            .body(body)
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let h = resp.headers_mut();
        if let Ok(v) = HeaderValue::from_str(&mime) {
            h.insert(header::CONTENT_TYPE, v);
        }
        if let Ok(v) = HeaderValue::from_str(&length.to_string()) {
            h.insert(header::CONTENT_LENGTH, v);
        }
        if let Ok(v) = HeaderValue::from_str(&format!("bytes {start}-{end}/{file_len}")) {
            h.insert(header::CONTENT_RANGE, v);
        }
        h.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
        h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        return Ok(resp);
    }

    let body = Body::from_stream(ReaderStream::new(stream));
    let mut resp = Response::builder()
        .status(StatusCode::OK)
        .body(body)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let h = resp.headers_mut();
    if let Ok(v) = HeaderValue::from_str(&mime) {
        h.insert(header::CONTENT_TYPE, v);
    }
    if let Ok(v) = HeaderValue::from_str(&file_len.to_string()) {
        h.insert(header::CONTENT_LENGTH, v);
    }
    h.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(resp)
}

#[cfg(target_os = "android")]
async fn stream_serve(
    State(_state): State<CloudState>,
    AxumPath((_info_hash, _file_index)): AxumPath<(String, usize)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    Err(err(
        StatusCode::SERVICE_UNAVAILABLE,
        "torrent streaming unavailable on this platform",
    ))
}

