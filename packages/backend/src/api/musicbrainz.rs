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
    Router::new().route("/recording/{id}", get(get_recording))
}

#[derive(Deserialize)]
struct RecordingQuery {
    refresh: Option<String>,
}

async fn get_recording(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<RecordingQuery>,
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
