use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

const MB_BASE: &str = "https://musicbrainz.org/ws/2";
const USER_AGENT: &str = "mhaol/1.0.0 (https://github.com/arktosmos/mhaol)";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/recording/{id}", get(get_recording))
        .route("/release-group/{id}", get(get_release_group))
        .route("/release/{id}", get(get_release))
        .route("/popular", get(get_popular))
        .route("/popular-artists", get(get_popular_artists))
}

#[derive(Deserialize)]
struct MbQuery {
    refresh: Option<String>,
}

#[derive(Deserialize)]
struct PopularQuery {
    genre: Option<String>,
}

async fn get_recording(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<MbQuery>,
) -> impl IntoResponse {
    if id.len() != 36 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid recording ID" })),
        )
            .into_response();
    }

    let refresh = query.refresh.as_deref() == Some("true");

    // Check cache
    if !refresh {
        let conn = state.db.lock();
        if let Ok(data) = conn.query_row(
            "SELECT data FROM musicbrainz_recordings WHERE mbid = ?1",
            rusqlite::params![id],
            |row| row.get::<_, String>(0),
        ) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Json(parsed).into_response();
            }
        }
    }

    let url = format!(
        "{}/recording/{}?inc=artist-credits+releases+release-groups&fmt=json",
        MB_BASE, id
    );

    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<serde_json::Value>().await {
                Ok(data) => {
                    let data_str = serde_json::to_string(&data).unwrap_or_default();
                    let conn = state.db.lock();
                    let _ = conn.execute(
                        "INSERT INTO musicbrainz_recordings (mbid, data) VALUES (?1, ?2)
                         ON CONFLICT(mbid) DO UPDATE SET data = ?2, fetched_at = datetime('now')",
                        rusqlite::params![id, data_str],
                    );
                    Json(data).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        Ok(resp) if resp.status() == reqwest::StatusCode::NOT_FOUND => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Recording not found" })),
        )
            .into_response(),
        _ => {
            // Try stale cache
            let conn = state.db.lock();
            if let Ok(data) = conn.query_row(
                "SELECT data FROM musicbrainz_recordings WHERE mbid = ?1",
                rusqlite::params![id],
                |row| row.get::<_, String>(0),
            ) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "MusicBrainz API unavailable" })),
            )
                .into_response()
        }
    }
}

async fn get_release_group(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<MbQuery>,
) -> impl IntoResponse {
    if id.len() != 36 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid release group ID" })),
        )
            .into_response();
    }

    let refresh = query.refresh.as_deref() == Some("true");

    if !refresh {
        let conn = state.db.lock();
        if let Ok(data) = conn.query_row(
            "SELECT data FROM musicbrainz_release_groups WHERE mbid = ?1",
            rusqlite::params![id],
            |row| row.get::<_, String>(0),
        ) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Json(parsed).into_response();
            }
        }
    }

    let url = format!(
        "{}/release-group/{}?inc=artist-credits+releases&fmt=json",
        MB_BASE, id
    );

    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<serde_json::Value>().await {
                Ok(data) => {
                    let data_str = serde_json::to_string(&data).unwrap_or_default();
                    let conn = state.db.lock();
                    let _ = conn.execute(
                        "INSERT INTO musicbrainz_release_groups (mbid, data) VALUES (?1, ?2)
                         ON CONFLICT(mbid) DO UPDATE SET data = ?2, fetched_at = datetime('now')",
                        rusqlite::params![id, data_str],
                    );
                    Json(data).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        Ok(resp) if resp.status() == reqwest::StatusCode::NOT_FOUND => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Release group not found" })),
        )
            .into_response(),
        _ => {
            let conn = state.db.lock();
            if let Ok(data) = conn.query_row(
                "SELECT data FROM musicbrainz_release_groups WHERE mbid = ?1",
                rusqlite::params![id],
                |row| row.get::<_, String>(0),
            ) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "MusicBrainz API unavailable" })),
            )
                .into_response()
        }
    }
}

async fn get_release(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<MbQuery>,
) -> impl IntoResponse {
    if id.len() != 36 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid release ID" })),
        )
            .into_response();
    }

    let refresh = query.refresh.as_deref() == Some("true");

    if !refresh {
        let conn = state.db.lock();
        if let Ok(data) = conn.query_row(
            "SELECT data FROM musicbrainz_releases WHERE mbid = ?1",
            rusqlite::params![id],
            |row| row.get::<_, String>(0),
        ) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Json(parsed).into_response();
            }
        }
    }

    let url = format!(
        "{}/release/{}?inc=recordings+artist-credits+media&fmt=json",
        MB_BASE, id
    );

    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<serde_json::Value>().await {
                Ok(data) => {
                    let data_str = serde_json::to_string(&data).unwrap_or_default();
                    let conn = state.db.lock();
                    let _ = conn.execute(
                        "INSERT INTO musicbrainz_releases (mbid, data) VALUES (?1, ?2)
                         ON CONFLICT(mbid) DO UPDATE SET data = ?2, fetched_at = datetime('now')",
                        rusqlite::params![id, data_str],
                    );
                    Json(data).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        Ok(resp) if resp.status() == reqwest::StatusCode::NOT_FOUND => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Release not found" })),
        )
            .into_response(),
        _ => {
            let conn = state.db.lock();
            if let Ok(data) = conn.query_row(
                "SELECT data FROM musicbrainz_releases WHERE mbid = ?1",
                rusqlite::params![id],
                |row| row.get::<_, String>(0),
            ) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "MusicBrainz API unavailable" })),
            )
                .into_response()
        }
    }
}

const POPULAR_GENRES: &[&str] = &[
    "rock", "pop", "electronic", "hip hop", "jazz", "classical",
    "r&b", "metal", "folk", "soul", "punk", "blues", "country",
    "ambient", "indie", "alternative",
];

async fn get_popular(
    State(state): State<AppState>,
    Query(query): Query<PopularQuery>,
) -> impl IntoResponse {
    let genre = query.genre.as_deref().unwrap_or("rock");

    // Validate genre against allowed list
    let genre_lower = genre.to_lowercase();
    if !POPULAR_GENRES.contains(&genre_lower.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid genre", "allowed": POPULAR_GENRES })),
        )
            .into_response();
    }

    // Check cache (valid for 24 hours)
    {
        let conn = state.db.lock();
        if let Ok(data) = conn.query_row(
            "SELECT data FROM musicbrainz_popular_cache WHERE genre = ?1 AND fetched_at > datetime('now', '-24 hours')",
            rusqlite::params![genre_lower],
            |row| row.get::<_, String>(0),
        ) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Json(parsed).into_response();
            }
        }
    }

    let search_query = format!("tag:{} AND primarytype:album AND status:official", genre_lower);
    let url = format!(
        "{}/release-group?query={}&fmt=json&limit=30",
        MB_BASE,
        urlencoding::encode(&search_query)
    );

    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<serde_json::Value>().await {
                Ok(data) => {
                    let data_str = serde_json::to_string(&data).unwrap_or_default();
                    let conn = state.db.lock();
                    let _ = conn.execute(
                        "INSERT INTO musicbrainz_popular_cache (genre, data) VALUES (?1, ?2)
                         ON CONFLICT(genre) DO UPDATE SET data = ?2, fetched_at = datetime('now')",
                        rusqlite::params![genre_lower, data_str],
                    );
                    Json(data).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        _ => {
            // Try stale cache
            let conn = state.db.lock();
            if let Ok(data) = conn.query_row(
                "SELECT data FROM musicbrainz_popular_cache WHERE genre = ?1",
                rusqlite::params![genre_lower],
                |row| row.get::<_, String>(0),
            ) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "MusicBrainz API unavailable" })),
            )
                .into_response()
        }
    }
}

async fn get_popular_artists(
    State(state): State<AppState>,
    Query(query): Query<PopularQuery>,
) -> impl IntoResponse {
    let genre = query.genre.as_deref().unwrap_or("rock");

    let genre_lower = genre.to_lowercase();
    if !POPULAR_GENRES.contains(&genre_lower.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid genre", "allowed": POPULAR_GENRES })),
        )
            .into_response();
    }

    // Check cache (valid for 24 hours)
    {
        let conn = state.db.lock();
        if let Ok(data) = conn.query_row(
            "SELECT data FROM musicbrainz_popular_artists_cache WHERE genre = ?1 AND fetched_at > datetime('now', '-24 hours')",
            rusqlite::params![genre_lower],
            |row| row.get::<_, String>(0),
        ) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Json(parsed).into_response();
            }
        }
    }

    let search_query = format!("tag:{}", genre_lower);
    let url = format!(
        "{}/artist?query={}&fmt=json&limit=30",
        MB_BASE,
        urlencoding::encode(&search_query)
    );

    let client = reqwest::Client::new();
    match client
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<serde_json::Value>().await {
                Ok(data) => {
                    let data_str = serde_json::to_string(&data).unwrap_or_default();
                    let conn = state.db.lock();
                    let _ = conn.execute(
                        "INSERT INTO musicbrainz_popular_artists_cache (genre, data) VALUES (?1, ?2)
                         ON CONFLICT(genre) DO UPDATE SET data = ?2, fetched_at = datetime('now')",
                        rusqlite::params![genre_lower, data_str],
                    );
                    Json(data).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        _ => {
            let conn = state.db.lock();
            if let Ok(data) = conn.query_row(
                "SELECT data FROM musicbrainz_popular_artists_cache WHERE genre = ?1",
                rusqlite::params![genre_lower],
                |row| row.get::<_, String>(0),
            ) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "MusicBrainz API unavailable" })),
            )
                .into_response()
        }
    }
}
