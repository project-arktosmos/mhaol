use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Response, Sse,
    },
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/config", get(get_config).put(set_config))
        .route("/debug", get(debug_info))
        .route("/search", get(search_torrents))
        .route("/status", get(get_status))
        .route("/storage/clear", post(clear_storage))
        .route("/torrents", get(list_torrents).post(add_torrent))
        .route("/torrents/{info_hash}", delete(remove_torrent))
        .route("/torrents/{info_hash}/pause", post(pause_torrent))
        .route("/torrents/{info_hash}/resume", post(resume_torrent))
        .route("/torrents/events", get(torrent_events))
        .route("/torrents/remove-all", post(remove_all))
        .route("/torrents/{info_hash}/files", get(list_torrent_files))
        .route(
            "/torrents/{info_hash}/stream/{file_idx}",
            get(stream_torrent_file),
        )
        .route("/torrents/{info_hash}/stream", get(stream_torrent_largest))
        .route(
            "/torrents/{info_hash}/stream/start",
            post(start_streaming),
        )
        .route(
            "/torrents/{info_hash}/stream/stop",
            post(stop_streaming),
        )
}

async fn get_config(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "downloadPath": state.torrent_manager.download_path(),
        "initialized": state.torrent_manager.is_initialized(),
    }))
}

async fn set_config(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let library_id = body["library_id"].as_str().unwrap_or("");
    if library_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Missing library_id" })),
        )
            .into_response();
    }

    match state.libraries.get(library_id) {
        Some(library) => {
            let path = std::path::PathBuf::from(&library.path);
            state.torrent_manager.set_download_path(path);
            Json(serde_json::json!({
                "download_path": state.torrent_manager.download_path(),
            }))
            .into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Library not found" })),
        )
            .into_response(),
    }
}

async fn debug_info(State(state): State<AppState>) -> impl IntoResponse {
    match state.torrent_manager.debug_info().await {
        Ok(info) => Json(serde_json::json!({ "debug": info })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    let initialized = state.torrent_manager.is_initialized();
    let download_path = state.torrent_manager.download_path();
    let stats = state.torrent_manager.stats().await.ok();

    Json(serde_json::json!({
        "initialized": initialized,
        "download_path": download_path,
        "stats": stats,
    }))
    .into_response()
}

async fn clear_storage(State(state): State<AppState>) -> impl IntoResponse {
    match state.torrent_manager.clear_storage().await {
        Ok(()) => Json(serde_json::json!({ "ok": true })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn list_torrents(State(state): State<AppState>) -> impl IntoResponse {
    match state.torrent_manager.list().await {
        Ok(torrents) => Json(serde_json::to_value(torrents).unwrap()).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn add_torrent(
    State(state): State<AppState>,
    Json(body): Json<mhaol_torrent::AddTorrentRequest>,
) -> impl IntoResponse {
    match state.torrent_manager.add(body).await {
        Ok(info) => {
            let state_str = format!("{:?}", info.state).to_lowercase();
            state.torrent_downloads.upsert(
                &info.info_hash,
                &info.name,
                info.size as i64,
                info.progress,
                &state_str,
                info.download_speed as i64,
                info.upload_speed as i64,
                info.peers as i64,
                info.seeds as i64,
                info.added_at,
                info.eta.map(|e| e as i64),
                info.output_path.as_deref(),
                "magnet",
            );
            (
                StatusCode::CREATED,
                Json(serde_json::to_value(info).unwrap()),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn remove_torrent(
    State(state): State<AppState>,
    Path(info_hash): Path<String>,
) -> impl IntoResponse {
    match state.torrent_manager.list().await {
        Ok(torrents) => {
            if let Some(torrent) = torrents.iter().find(|t| t.info_hash == info_hash) {
                if let Err(e) = state.torrent_manager.remove(torrent.id).await {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": e.to_string() })),
                    )
                        .into_response();
                }
            }
            state.torrent_downloads.delete(&info_hash);
            Json(serde_json::json!({ "ok": true })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn pause_torrent(
    State(state): State<AppState>,
    Path(info_hash): Path<String>,
) -> impl IntoResponse {
    match find_torrent_id(&state, &info_hash).await {
        Some(id) => match state.torrent_manager.pause(id).await {
            Ok(()) => {
                if let Ok(torrents) = state.torrent_manager.list().await {
                    if let Some(t) = torrents.iter().find(|t| t.info_hash == info_hash) {
                        let state_str = format!("{:?}", t.state).to_lowercase();
                        state.torrent_downloads.update_state(
                            &info_hash,
                            t.progress,
                            &state_str,
                            t.download_speed as i64,
                            t.upload_speed as i64,
                            t.peers as i64,
                            t.seeds as i64,
                            t.eta.map(|e| e as i64),
                            t.output_path.as_deref(),
                        );
                    }
                }
                Json(serde_json::json!({ "ok": true })).into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response(),
        },
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Torrent not found" })),
        )
            .into_response(),
    }
}

async fn resume_torrent(
    State(state): State<AppState>,
    Path(info_hash): Path<String>,
) -> impl IntoResponse {
    match find_torrent_id(&state, &info_hash).await {
        Some(id) => match state.torrent_manager.resume(id).await {
            Ok(()) => {
                if let Ok(torrents) = state.torrent_manager.list().await {
                    if let Some(t) = torrents.iter().find(|t| t.info_hash == info_hash) {
                        let state_str = format!("{:?}", t.state).to_lowercase();
                        state.torrent_downloads.update_state(
                            &info_hash,
                            t.progress,
                            &state_str,
                            t.download_speed as i64,
                            t.upload_speed as i64,
                            t.peers as i64,
                            t.seeds as i64,
                            t.eta.map(|e| e as i64),
                            t.output_path.as_deref(),
                        );
                    }
                }
                Json(serde_json::json!({ "ok": true })).into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response(),
        },
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Torrent not found" })),
        )
            .into_response(),
    }
}

async fn remove_all(State(state): State<AppState>) -> impl IntoResponse {
    match state.torrent_manager.remove_all().await {
        Ok(count) => {
            state.torrent_downloads.delete_all();
            Json(serde_json::json!({ "ok": true, "removed": count })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn torrent_events(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let torrent_manager = state.torrent_manager.clone();
    let torrent_downloads = state.torrent_downloads.clone();

    let stream = async_stream::stream! {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
        loop {
            interval.tick().await;
            if let Ok(torrents) = torrent_manager.list().await {
                for t in &torrents {
                    let state_str = format!("{:?}", t.state).to_lowercase();
                    torrent_downloads.upsert(
                        &t.info_hash,
                        &t.name,
                        t.size as i64,
                        t.progress,
                        &state_str,
                        t.download_speed as i64,
                        t.upload_speed as i64,
                        t.peers as i64,
                        t.seeds as i64,
                        t.added_at,
                        t.eta.map(|e| e as i64),
                        t.output_path.as_deref(),
                        "magnet",
                    );
                }
                if let Ok(json) = serde_json::to_string(&torrents) {
                    yield Ok(Event::default().event("torrents").data(json));
                }
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

const TRACKERS: &[&str] = &[
    "udp://tracker.opentrackr.org:1337/announce",
    "udp://tracker.openbittorrent.com:6969/announce",
    "udp://open.stealth.si:80/announce",
    "udp://tracker.torrent.eu.org:451/announce",
    "udp://tracker.dler.org:6969/announce",
    "udp://opentracker.i2p.rocks:6969/announce",
];

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
    cat: Option<String>,
}

#[derive(Serialize)]
struct SearchResult {
    id: String,
    name: String,
    #[serde(rename = "infoHash")]
    info_hash: String,
    seeders: i64,
    leechers: i64,
    size: i64,
    #[serde(rename = "fileCount")]
    file_count: i64,
    #[serde(rename = "uploadedBy")]
    uploaded_by: String,
    #[serde(rename = "uploadedAt")]
    uploaded_at: i64,
    category: String,
    #[serde(rename = "magnetLink")]
    magnet_uri: String,
}

async fn search_torrents(
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let q = match &query.q {
        Some(q) if !q.is_empty() => q,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing query parameter 'q'" })),
            )
                .into_response()
        }
    };

    let cat = query.cat.as_deref().unwrap_or("0");
    let url = format!(
        "https://apibay.org/q.php?q={}&cat={}",
        urlencoding::encode(q),
        cat
    );

    match reqwest::get(&url).await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<Vec<serde_json::Value>>().await {
                Ok(results) => {
                    // Check for "no results" response
                    if results.len() == 1 {
                        if let Some(name) = results[0]["name"].as_str() {
                            if name == "No results returned" {
                                return Json(serde_json::json!([])).into_response();
                            }
                        }
                    }

                    let search_results: Vec<SearchResult> = results
                        .iter()
                        .filter_map(|r| {
                            let info_hash = r["info_hash"].as_str()?.to_string();
                            let name = r["name"].as_str()?.to_string();

                            // Build magnet URI
                            let trackers: String = TRACKERS
                                .iter()
                                .map(|t| format!("&tr={}", urlencoding::encode(t)))
                                .collect();
                            let magnet_uri = format!(
                                "magnet:?xt=urn:btih:{}&dn={}{}",
                                info_hash,
                                urlencoding::encode(&name),
                                trackers
                            );

                            Some(SearchResult {
                                id: r["id"].as_str().unwrap_or("0").to_string(),
                                name,
                                info_hash,
                                seeders: r["seeders"].as_str()?.parse().ok()?,
                                leechers: r["leechers"].as_str()?.parse().ok()?,
                                size: r["size"].as_str()?.parse().ok()?,
                                file_count: r["num_files"].as_str()?.parse().unwrap_or(0),
                                uploaded_by: r["username"].as_str().unwrap_or("").to_string(),
                                uploaded_at: r["added"].as_str()?.parse().ok()?,
                                category: r["category"].as_str().unwrap_or("0").to_string(),
                                magnet_uri,
                            })
                        })
                        .collect();

                    Json(serde_json::to_value(search_results).unwrap()).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "PirateBay API unavailable" })),
        )
            .into_response(),
    }
}

async fn list_torrent_files(
    State(state): State<AppState>,
    Path(info_hash): Path<String>,
) -> impl IntoResponse {
    let torrent_id = match find_torrent_id(&state, &info_hash).await {
        Some(id) => id,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Torrent not found" })),
            )
                .into_response()
        }
    };

    match state.torrent_manager.list_files(torrent_id).await {
        Ok(files) => Json(serde_json::to_value(files).unwrap()).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn resolve_stream_url(
    state: &AppState,
    info_hash: &str,
    file_idx: usize,
) -> Result<String, Response> {
    let torrent_id = find_torrent_id(state, info_hash).await.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Torrent not found" })),
        )
            .into_response()
    })?;

    let http_api_addr = state.torrent_manager.get_http_api_addr().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Torrent HTTP API not available" })),
        )
            .into_response()
    })?;

    Ok(mhaol_http_stream::build_stream_url(
        &http_api_addr,
        torrent_id,
        file_idx,
    ))
}

async fn stream_torrent_file(
    State(state): State<AppState>,
    Path((info_hash, file_idx)): Path<(String, usize)>,
    headers: HeaderMap,
) -> Response {
    let range = headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok());
    match resolve_stream_url(&state, &info_hash, file_idx).await {
        Ok(url) => mhaol_http_stream::proxy_stream(&url, range).await,
        Err(resp) => resp,
    }
}

async fn stream_torrent_largest(
    State(state): State<AppState>,
    Path(info_hash): Path<String>,
    headers: HeaderMap,
) -> Response {
    let torrent_id = match find_torrent_id(&state, &info_hash).await {
        Some(id) => id,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Torrent not found" })),
            )
                .into_response()
        }
    };

    let file_idx = match state.torrent_manager.list_files(torrent_id).await {
        Ok(files) if !files.is_empty() => {
            files
                .iter()
                .max_by_key(|f| f.size)
                .map(|f| f.id)
                .unwrap_or(0)
        }
        Ok(_) => 0,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    let range = headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok());
    match resolve_stream_url(&state, &info_hash, file_idx).await {
        Ok(url) => mhaol_http_stream::proxy_stream(&url, range).await,
        Err(resp) => resp,
    }
}

async fn start_streaming(
    State(state): State<AppState>,
    Path(info_hash): Path<String>,
) -> impl IntoResponse {
    if let Err(e) = state.torrent_manager.pause_all_except(&info_hash).await {
        tracing::warn!("Failed to pause other torrents: {}", e);
    }

    let stream_url = format!("/api/torrent/torrents/{}/stream", info_hash);
    Json(serde_json::json!({ "streamUrl": stream_url }))
}

async fn stop_streaming(State(state): State<AppState>) -> impl IntoResponse {
    if let Err(e) = state.torrent_manager.resume_auto_paused().await {
        tracing::warn!("Failed to resume auto-paused torrents: {}", e);
    }

    Json(serde_json::json!({ "ok": true }))
}

async fn find_torrent_id(state: &AppState, info_hash: &str) -> Option<usize> {
    state
        .torrent_manager
        .list()
        .await
        .ok()?
        .iter()
        .find(|t| t.info_hash == info_hash)
        .map(|t| t.id)
}
