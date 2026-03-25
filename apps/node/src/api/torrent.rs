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
        .route("/fetch-cache/hashes", get(list_fetch_cache_hashes))
        .route("/fetch-cache/summaries", get(list_fetch_cache_summaries))
        .route("/fetch-cache/{tmdb_id}", get(get_fetch_cache).delete(delete_fetch_cache))
        .route("/fetch-cache", get(list_fetch_cache_ids).post(save_fetch_cache))
        .route("/tv-fetch-cache/{tmdb_id}", get(get_tv_fetch_cache).delete(delete_tv_fetch_cache))
        .route("/tv-fetch-cache", post(save_tv_fetch_cache))
        .route("/music-fetch-cache/hashes", get(list_music_fetch_cache_hashes))
        .route("/music-fetch-cache/{musicbrainz_id}", get(get_music_fetch_cache).delete(delete_music_fetch_cache))
        .route("/music-fetch-cache", post(save_music_fetch_cache))
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
    let output_path = state.downloads.get(&info_hash).and_then(|row| {
        match (&row.output_path, &row.name) {
            (Some(p), name) if !name.is_empty() => Some(format!("{}/{}", p, name)),
            (Some(p), _) => Some(p.clone()),
            _ => None,
        }
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

// --- Fetch cache endpoints ---

async fn list_fetch_cache_ids(State(state): State<AppState>) -> impl IntoResponse {
    Json(state.torrent_fetch_cache.get_all_tmdb_ids())
}

async fn list_fetch_cache_summaries(State(state): State<AppState>) -> impl IntoResponse {
    let entries: Vec<serde_json::Value> = state
        .torrent_fetch_cache
        .get_all_summaries()
        .into_iter()
        .map(|(tmdb_id, name)| serde_json::json!({ "tmdbId": tmdb_id, "name": name }))
        .collect();
    Json(entries)
}

async fn list_fetch_cache_hashes(State(state): State<AppState>) -> impl IntoResponse {
    let entries: Vec<serde_json::Value> = state
        .torrent_fetch_cache
        .get_all_info_hashes()
        .into_iter()
        .map(|(tmdb_id, info_hash)| {
            serde_json::json!({ "tmdbId": tmdb_id, "infoHash": info_hash })
        })
        .collect();
    Json(entries)
}

async fn get_fetch_cache(
    State(state): State<AppState>,
    Path(tmdb_id): Path<i64>,
) -> impl IntoResponse {
    match state.torrent_fetch_cache.get(tmdb_id) {
        Some(row) => Json(serde_json::json!({
            "tmdbId": row.tmdb_id,
            "mediaType": row.media_type,
            "candidate": serde_json::from_str::<serde_json::Value>(&row.candidate_json).unwrap_or_default(),
            "createdAt": row.created_at,
        }))
        .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveFetchCacheBody {
    tmdb_id: i64,
    media_type: String,
    candidate: serde_json::Value,
}

async fn save_fetch_cache(
    State(state): State<AppState>,
    Json(body): Json<SaveFetchCacheBody>,
) -> impl IntoResponse {
    let candidate_json = serde_json::to_string(&body.candidate).unwrap_or_default();
    state
        .torrent_fetch_cache
        .upsert(body.tmdb_id, &body.media_type, &candidate_json);
    StatusCode::CREATED
}

async fn delete_fetch_cache(
    State(state): State<AppState>,
    Path(tmdb_id): Path<i64>,
) -> impl IntoResponse {
    state.torrent_fetch_cache.delete(tmdb_id);
    StatusCode::NO_CONTENT
}

// --- TV Fetch cache endpoints ---

async fn get_tv_fetch_cache(
    State(state): State<AppState>,
    Path(tmdb_id): Path<i64>,
) -> impl IntoResponse {
    let rows = state.tv_torrent_fetch_cache.get_for_show(tmdb_id);
    let entries: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.id,
                "tmdbId": row.tmdb_id,
                "scope": row.scope,
                "seasonNumber": row.season_number,
                "episodeNumber": row.episode_number,
                "candidate": serde_json::from_str::<serde_json::Value>(&row.candidate_json).unwrap_or_default(),
                "createdAt": row.created_at,
            })
        })
        .collect();
    Json(entries)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveTvFetchCacheBody {
    tmdb_id: i64,
    scope: String,
    season_number: Option<i64>,
    episode_number: Option<i64>,
    candidate: serde_json::Value,
}

async fn save_tv_fetch_cache(
    State(state): State<AppState>,
    Json(body): Json<SaveTvFetchCacheBody>,
) -> impl IntoResponse {
    let candidate_json = serde_json::to_string(&body.candidate).unwrap_or_default();
    state.tv_torrent_fetch_cache.upsert(
        body.tmdb_id,
        &body.scope,
        body.season_number,
        body.episode_number,
        &candidate_json,
    );
    StatusCode::CREATED
}

async fn delete_tv_fetch_cache(
    State(state): State<AppState>,
    Path(tmdb_id): Path<i64>,
) -> impl IntoResponse {
    state.tv_torrent_fetch_cache.delete_for_show(tmdb_id);
    StatusCode::NO_CONTENT
}

// --- Music Fetch cache endpoints ---

async fn list_music_fetch_cache_hashes(State(state): State<AppState>) -> impl IntoResponse {
    let entries: Vec<serde_json::Value> = state
        .music_torrent_fetch_cache
        .get_all_info_hashes()
        .into_iter()
        .map(|(musicbrainz_id, info_hash)| {
            serde_json::json!({ "musicbrainzId": musicbrainz_id, "infoHash": info_hash })
        })
        .collect();
    Json(entries)
}

async fn get_music_fetch_cache(
    State(state): State<AppState>,
    Path(musicbrainz_id): Path<String>,
) -> impl IntoResponse {
    let rows = state.music_torrent_fetch_cache.get_for_id(&musicbrainz_id);
    let entries: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.id,
                "musicbrainzId": row.musicbrainz_id,
                "scope": row.scope,
                "candidate": serde_json::from_str::<serde_json::Value>(&row.candidate_json).unwrap_or_default(),
                "createdAt": row.created_at,
            })
        })
        .collect();
    Json(entries)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveMusicFetchCacheBody {
    musicbrainz_id: String,
    scope: String,
    candidate: serde_json::Value,
}

async fn save_music_fetch_cache(
    State(state): State<AppState>,
    Json(body): Json<SaveMusicFetchCacheBody>,
) -> impl IntoResponse {
    let candidate_json = serde_json::to_string(&body.candidate).unwrap_or_default();
    state.music_torrent_fetch_cache.upsert(
        &body.musicbrainz_id,
        &body.scope,
        &candidate_json,
    );
    StatusCode::CREATED
}

async fn delete_music_fetch_cache(
    State(state): State<AppState>,
    Path(musicbrainz_id): Path<String>,
) -> impl IntoResponse {
    state.music_torrent_fetch_cache.delete_for_id(&musicbrainz_id);
    StatusCode::NO_CONTENT
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
