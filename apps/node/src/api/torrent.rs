use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Sse,
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
        Ok(mut torrents) => {
            let live_hashes: std::collections::HashSet<String> =
                torrents.iter().map(|t| t.info_hash.clone()).collect();
            for row in state.downloads.get_by_type("torrent") {
                if live_hashes.contains(&row.id) {
                    continue;
                }
                let torrent_state = match row.state.as_str() {
                    "initializing" => mhaol_torrent::TorrentState::Initializing,
                    "downloading" => mhaol_torrent::TorrentState::Downloading,
                    "seeding" => mhaol_torrent::TorrentState::Seeding,
                    "paused" => mhaol_torrent::TorrentState::Paused,
                    "checking" => mhaol_torrent::TorrentState::Checking,
                    _ => mhaol_torrent::TorrentState::Paused,
                };
                torrents.push(mhaol_torrent::TorrentInfo {
                    id: 0,
                    info_hash: row.id,
                    name: row.name,
                    size: row.size as u64,
                    progress: row.progress,
                    download_speed: 0,
                    upload_speed: 0,
                    peers: 0,
                    seeds: 0,
                    state: torrent_state,
                    added_at: row.added_at.unwrap_or(0),
                    eta: None,
                    output_path: row.output_path,
                });
            }
            Json(serde_json::to_value(torrents).unwrap()).into_response()
        }
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
    if let Some(ref path) = body.download_path {
        if let Err(e) = std::fs::create_dir_all(path) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to create download directory: {}", e) })),
            )
                .into_response();
        }
    }

    match state.torrent_manager.add(body).await {
        Ok(info) => {
            let state_str = format!("{:?}", info.state).to_lowercase();
            state.downloads.upsert_torrent(
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

#[derive(Deserialize)]
struct RemoveQuery {
    #[serde(default)]
    delete_files: bool,
}

async fn remove_torrent(
    State(state): State<AppState>,
    Path(info_hash): Path<String>,
    Query(query): Query<RemoveQuery>,
) -> impl IntoResponse {
    // Grab output_path before removing, so we can delete files if requested
    let output_path =
        state
            .downloads
            .get(&info_hash)
            .and_then(|row| match (&row.output_path, &row.name) {
                (Some(p), name) if !name.is_empty() => Some(format!("{}/{}", p, name)),
                (Some(p), _) => Some(p.clone()),
                _ => None,
            });

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
            state.downloads.delete(&info_hash);

            if query.delete_files {
                if let Some(path) = output_path {
                    let p = std::path::Path::new(&path);
                    if p.is_dir() {
                        let _ = std::fs::remove_dir_all(p);
                    } else if p.is_file() {
                        let _ = std::fs::remove_file(p);
                    }
                }
            }

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
                        state.downloads.update_torrent_state(
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
                        state.downloads.update_torrent_state(
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
            state.downloads.delete_all_by_type("torrent");
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
    let downloads = state.downloads.clone();

    let stream = async_stream::stream! {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
        loop {
            interval.tick().await;
            if let Ok(mut torrents) = torrent_manager.list().await {
                for t in &torrents {
                    let state_str = format!("{:?}", t.state).to_lowercase();
                    downloads.upsert_torrent(
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

                // Merge DB-only records (not currently live in the engine)
                let live_hashes: std::collections::HashSet<String> =
                    torrents.iter().map(|t| t.info_hash.clone()).collect();
                let db_rows = downloads.get_by_type("torrent");
                for row in db_rows {
                    if live_hashes.contains(&row.id) {
                        continue;
                    }
                    let state = match row.state.as_str() {
                        "initializing" => mhaol_torrent::TorrentState::Initializing,
                        "downloading" => mhaol_torrent::TorrentState::Downloading,
                        "seeding" => mhaol_torrent::TorrentState::Seeding,
                        "paused" => mhaol_torrent::TorrentState::Paused,
                        "checking" => mhaol_torrent::TorrentState::Checking,
                        _ => mhaol_torrent::TorrentState::Paused,
                    };
                    torrents.push(mhaol_torrent::TorrentInfo {
                        id: 0,
                        info_hash: row.id,
                        name: row.name,
                        size: row.size as u64,
                        progress: row.progress,
                        download_speed: 0,
                        upload_speed: 0,
                        peers: 0,
                        seeds: 0,
                        state,
                        added_at: row.added_at.unwrap_or(0),
                        eta: None,
                        output_path: row.output_path,
                    });
                }

                // Auto-resume queued torrents if under the concurrency limit
                let downloading_count = torrents.iter()
                    .filter(|t| matches!(t.state, mhaol_torrent::TorrentState::Downloading))
                    .count();
                if downloading_count < 10 {
                    let slots = 10 - downloading_count;
                    let to_resume: Vec<usize> = torrents.iter()
                        .filter(|t| matches!(t.state, mhaol_torrent::TorrentState::Paused)
                            && t.id > 0)
                        .take(slots)
                        .map(|t| t.id)
                        .collect();
                    for id in to_resume {
                        let _ = torrent_manager.resume(id).await;
                    }
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
    /// Language hint: `en` (default) hits PirateBay only; `es` additionally
    /// runs Spanish-enriched PirateBay queries and the configured Spanish
    /// indexers in parallel.
    lang: Option<String>,
}

#[derive(Serialize, Clone)]
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
    /// Source identifier — `piratebay` or one of the Spanish indexer ids.
    #[serde(skip_serializing_if = "Option::is_none")]
    indexer: Option<String>,
}

fn build_magnet(info_hash: &str, name: &str) -> String {
    let trackers: String = TRACKERS
        .iter()
        .map(|t| format!("&tr={}", urlencoding::encode(t)))
        .collect();
    format!(
        "magnet:?xt=urn:btih:{}&dn={}{}",
        info_hash,
        urlencoding::encode(name),
        trackers
    )
}

async fn search_piratebay(query: &str, cat: &str) -> Result<Vec<SearchResult>, String> {
    let url = format!(
        "https://apibay.org/q.php?q={}&cat={}",
        urlencoding::encode(query),
        cat
    );

    let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("PirateBay API returned {}", resp.status()));
    }

    let results: Vec<serde_json::Value> = resp.json().await.map_err(|e| e.to_string())?;

    // PirateBay returns a single sentinel row when there are no matches.
    if results.len() == 1 {
        if let Some(name) = results[0]["name"].as_str() {
            if name == "No results returned" {
                return Ok(Vec::new());
            }
        }
    }

    let parsed: Vec<SearchResult> = results
        .iter()
        .filter_map(|r| {
            let info_hash = r["info_hash"].as_str()?.to_string();
            let name = r["name"].as_str()?.to_string();
            let magnet_uri = build_magnet(&info_hash, &name);
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
                indexer: Some("piratebay".to_string()),
            })
        })
        .collect();

    Ok(parsed)
}

/// Merge the second list into the first, keeping the highest-seeder copy
/// when an `info_hash` collides across indexers.
fn merge_dedup(into: &mut Vec<SearchResult>, more: Vec<SearchResult>) {
    use std::collections::HashMap;
    let mut by_hash: HashMap<String, SearchResult> = into
        .drain(..)
        .map(|r| (r.info_hash.clone(), r))
        .collect();
    for r in more {
        match by_hash.get(&r.info_hash) {
            Some(existing) if existing.seeders >= r.seeders => continue,
            _ => {
                by_hash.insert(r.info_hash.clone(), r);
            }
        }
    }
    *into = by_hash.into_values().collect();
}

async fn search_torrents(Query(query): Query<SearchQuery>) -> impl IntoResponse {
    let q = match &query.q {
        Some(q) if !q.is_empty() => q.clone(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing query parameter 'q'" })),
            )
                .into_response()
        }
    };

    let cat = query.cat.as_deref().unwrap_or("0").to_string();
    let lang = query.lang.as_deref().unwrap_or("en");

    if lang == "es" {
        let queries = super::torrent_spanish::build_piratebay_queries(&q);
        let pb_handles: Vec<_> = queries
            .into_iter()
            .map(|qv| {
                let cat = cat.clone();
                tokio::spawn(async move { search_piratebay(&qv, &cat).await })
            })
            .collect();

        let q_for_indexers = q.clone();
        let spanish_handle =
            tokio::spawn(async move { super::torrent_spanish::search_all(&q_for_indexers).await });

        let mut aggregated: Vec<SearchResult> = Vec::new();
        for h in pb_handles {
            if let Ok(Ok(rows)) = h.await {
                merge_dedup(&mut aggregated, rows);
            }
        }
        if let Ok(spanish_rows) = spanish_handle.await {
            let converted: Vec<SearchResult> = spanish_rows
                .into_iter()
                .map(|r| SearchResult {
                    id: r.id,
                    name: r.name,
                    info_hash: r.info_hash,
                    seeders: r.seeders,
                    leechers: r.leechers,
                    size: r.size,
                    file_count: 0,
                    uploaded_by: String::new(),
                    uploaded_at: r.uploaded_at,
                    category: r.category,
                    magnet_uri: r.magnet_uri,
                    indexer: Some(r.indexer),
                })
                .collect();
            merge_dedup(&mut aggregated, converted);
        }
        aggregated.sort_by(|a, b| b.seeders.cmp(&a.seeders));
        return Json(serde_json::to_value(aggregated).unwrap()).into_response();
    }

    match search_piratebay(&q, &cat).await {
        Ok(results) => Json(serde_json::to_value(results).unwrap()).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
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
