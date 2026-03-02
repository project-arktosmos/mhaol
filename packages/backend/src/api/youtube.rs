use crate::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new().route("/oembed", get(oembed))
}

#[derive(Deserialize)]
struct OembedQuery {
    #[serde(rename = "videoId")]
    video_id: Option<String>,
}

async fn oembed(
    State(state): State<AppState>,
    Query(query): Query<OembedQuery>,
) -> impl IntoResponse {
    let video_id = match &query.video_id {
        Some(id) if id.len() == 11 => id,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Missing or invalid videoId parameter" })),
            )
                .into_response()
        }
    };

    // Check cache
    {
        let conn = state.db.lock();
        if let Ok(data) = conn.query_row(
            "SELECT data FROM youtube_videos WHERE video_id = ?1",
            rusqlite::params![video_id],
            |row| row.get::<_, String>(0),
        ) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Json(parsed).into_response();
            }
        }
    }

    let url = format!(
        "https://www.youtube.com/oembed?url=https://www.youtube.com/watch?v={}&format=json",
        video_id
    );

    match reqwest::get(&url).await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<serde_json::Value>().await {
                Ok(data) => {
                    let data_str = serde_json::to_string(&data).unwrap_or_default();
                    let conn = state.db.lock();
                    let _ = conn.execute(
                        "INSERT INTO youtube_videos (video_id, data) VALUES (?1, ?2)
                         ON CONFLICT(video_id) DO UPDATE SET data = ?2, fetched_at = datetime('now')",
                        rusqlite::params![video_id, data_str],
                    );
                    Json(data).into_response()
                }
                Err(e) => (
                    StatusCode::BAD_GATEWAY,
                    Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        _ => (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": "YouTube oEmbed API unavailable" })),
        )
            .into_response(),
    }
}
