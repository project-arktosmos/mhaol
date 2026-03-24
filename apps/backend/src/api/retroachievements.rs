use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

const RA_BASE: &str = "https://retroachievements.org";
const USER_AGENT: &str = "mhaol/1.0.0 (https://github.com/arktosmos/mhaol)";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/games", get(get_game_list))
        .route("/games/{id}", get(get_game_details))
        .route("/image/{*path}", get(serve_ra_image))
}

#[derive(Deserialize)]
struct GameListQuery {
    console: Option<u32>,
}

fn ra_credentials(state: &AppState) -> Option<(String, String)> {
    let user = state.settings.get("ra.apiUser").unwrap_or_default();
    let key = state.settings.get("ra.apiKey").unwrap_or_default();
    if user.is_empty() || key.is_empty() {
        return None;
    }
    Some((user, key))
}

async fn get_game_list(
    State(state): State<AppState>,
    Query(query): Query<GameListQuery>,
) -> impl IntoResponse {
    let console_id = query.console.unwrap_or(5); // Default: GBA

    // Check cache (valid for 24 hours)
    {
        let conn = state.db.lock();
        if let Ok(data) = conn.query_row(
            "SELECT data FROM ra_game_list_cache WHERE console_id = ?1 AND fetched_at > datetime('now', '-24 hours')",
            rusqlite::params![console_id],
            |row| row.get::<_, String>(0),
        ) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Json(parsed).into_response();
            }
        }
    }

    let (user, key) = match ra_credentials(&state) {
        Some(creds) => creds,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "RetroAchievements credentials not configured" })),
            )
                .into_response();
        }
    };

    let url = format!(
        "{}/API/API_GetGameList.php?z={}&y={}&i={}&h=1",
        RA_BASE, user, key, console_id
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
                        "INSERT INTO ra_game_list_cache (console_id, data) VALUES (?1, ?2)
                         ON CONFLICT(console_id) DO UPDATE SET data = ?2, fetched_at = datetime('now')",
                        rusqlite::params![console_id, data_str],
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
                "SELECT data FROM ra_game_list_cache WHERE console_id = ?1",
                rusqlite::params![console_id],
                |row| row.get::<_, String>(0),
            ) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "RetroAchievements API unavailable" })),
            )
                .into_response()
        }
    }
}

async fn get_game_details(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> impl IntoResponse {
    // Check cache
    {
        let conn = state.db.lock();
        if let Ok(data) = conn.query_row(
            "SELECT data FROM ra_game_details_cache WHERE game_id = ?1",
            rusqlite::params![id],
            |row| row.get::<_, String>(0),
        ) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Json(parsed).into_response();
            }
        }
    }

    let (user, key) = match ra_credentials(&state) {
        Some(creds) => creds,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "RetroAchievements credentials not configured" })),
            )
                .into_response();
        }
    };

    let url = format!(
        "{}/API/API_GetGameExtended.php?z={}&y={}&i={}",
        RA_BASE, user, key, id
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
                        "INSERT INTO ra_game_details_cache (game_id, data) VALUES (?1, ?2)
                         ON CONFLICT(game_id) DO UPDATE SET data = ?2, fetched_at = datetime('now')",
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
            Json(serde_json::json!({ "error": "Game not found" })),
        )
            .into_response(),
        _ => {
            let conn = state.db.lock();
            if let Ok(data) = conn.query_row(
                "SELECT data FROM ra_game_details_cache WHERE game_id = ?1",
                rusqlite::params![id],
                |row| row.get::<_, String>(0),
            ) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                    return Json(parsed).into_response();
                }
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "RetroAchievements API unavailable" })),
            )
                .into_response()
        }
    }
}

/// GET /api/retroachievements/image/{*path} — serve RA images from disk cache.
async fn serve_ra_image(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    if path.contains("..") || !(path.starts_with("Images/") || path.starts_with("Badge/")) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Invalid image path" })),
        )
            .into_response();
    }

    let upstream_url = format!("https://media.retroachievements.org/{}", path);
    super::image_cache::serve_cached_image(
        &state.data_dir,
        "ra-images",
        &path,
        &upstream_url,
        604800,
    )
    .await
}
