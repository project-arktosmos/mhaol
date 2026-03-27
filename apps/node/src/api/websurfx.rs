use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

pub fn router() -> Router<crate::AppState> {
    Router::new().route("/search", get(search))
}

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
    max_results: Option<u32>,
}

async fn search(
    State(state): State<crate::AppState>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
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

    let max_results = query.max_results.unwrap_or(10);
    let cache_key = format!("{}:{}", q, max_results);

    // Check cache (1-hour TTL)
    {
        let conn = state.db.lock();
        if let Ok(data) = conn.query_row(
            "SELECT data FROM websurfx_search_cache WHERE cache_key = ?1 AND fetched_at > datetime('now', '-1 hours')",
            rusqlite::params![cache_key],
            |row| row.get::<_, String>(0),
        ) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                return Json(parsed).into_response();
            }
        }
    }

    // Cache miss — perform search
    let results = match state.websurfx.search(&q, Some(max_results)).await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response()
        }
    };

    let provider = state.websurfx.provider_name().to_string();
    let response = serde_json::json!({
        "results": results,
        "provider": provider,
    });

    // Store in cache
    {
        let conn = state.db.lock();
        let _ = conn.execute(
            "INSERT INTO websurfx_search_cache (cache_key, data) VALUES (?1, ?2)
             ON CONFLICT(cache_key) DO UPDATE SET data = ?2, fetched_at = datetime('now')",
            rusqlite::params![cache_key, serde_json::to_string(&response).unwrap_or_default()],
        );
    }

    Json(response).into_response()
}
