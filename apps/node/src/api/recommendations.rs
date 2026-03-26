use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FilterQuery {
    limit: Option<usize>,
    media_type: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/bulk", post(bulk_enqueue))
        .route("/status", get(status))
        .route("/top-movies", get(top_movies))
        .route("/top-movies-detail", get(top_movies_detail))
        .route("/top-genres", get(top_genres))
        .route("/{media_type}/{tmdb_id}", get(get_recommendations))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BulkItem {
    tmdb_id: i64,
    media_type: String,
}

#[derive(Deserialize)]
struct BulkRequest {
    items: Vec<BulkItem>,
}

async fn bulk_enqueue(
    State(state): State<AppState>,
    Json(body): Json<BulkRequest>,
) -> impl IntoResponse {
    let existing_tasks = state
        .queue
        .list(None, Some(mhaol_recommendations::TASK_FETCH));

    let mut enqueued = 0;
    for item in &body.items {
        if item.media_type != "movie" && item.media_type != "tv" {
            continue;
        }

        // Cancel/remove existing queue tasks for this tmdbId
        for task in &existing_tasks {
            if let Some(tid) = task.payload.get("tmdbId").and_then(|v| v.as_i64()) {
                let mt = task
                    .payload
                    .get("mediaType")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if tid == item.tmdb_id && mt == item.media_type {
                    let _ = state.queue.cancel(&task.id);
                    state.queue.remove(&task.id);
                }
            }
        }

        // Delete existing recommendation records
        state
            .recommendations
            .delete_for_source(item.tmdb_id, &item.media_type);

        state.queue.enqueue(
            mhaol_recommendations::TASK_FETCH,
            serde_json::json!({
                "tmdbId": item.tmdb_id,
                "mediaType": item.media_type,
            }),
        );
        enqueued += 1;
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "enqueued": enqueued })),
    )
}

async fn get_recommendations(
    State(state): State<AppState>,
    Path((media_type, tmdb_id)): Path<(String, i64)>,
) -> impl IntoResponse {
    let recs = state.recommendations.get_for_source(tmdb_id, &media_type);
    Json(serde_json::json!(recs))
}

async fn status(
    State(state): State<AppState>,
    Query(q): Query<FilterQuery>,
) -> impl IntoResponse {
    let all = state
        .queue
        .list(None, Some(mhaol_recommendations::TASK_FETCH));
    let filtered: Vec<_> = if let Some(ref mt) = q.media_type {
        all.into_iter()
            .filter(|t| {
                t.payload
                    .get("mediaType")
                    .and_then(|v| v.as_str())
                    .map(|v| v == mt)
                    .unwrap_or(false)
            })
            .collect()
    } else {
        all
    };
    let pending = filtered
        .iter()
        .filter(|t| t.status == mhaol_queue::QueueTaskStatus::Pending)
        .count();
    let running = filtered
        .iter()
        .filter(|t| t.status == mhaol_queue::QueueTaskStatus::Running)
        .count();
    let completed = filtered
        .iter()
        .filter(|t| t.status == mhaol_queue::QueueTaskStatus::Completed)
        .count();
    let failed = filtered
        .iter()
        .filter(|t| t.status == mhaol_queue::QueueTaskStatus::Failed)
        .count();
    Json(serde_json::json!({
        "pending": pending,
        "running": running,
        "completed": completed,
        "failed": failed,
        "total": filtered.len(),
    }))
}

async fn top_movies(
    State(state): State<AppState>,
    Query(q): Query<FilterQuery>,
) -> impl IntoResponse {
    let limit = q.limit.unwrap_or(50);
    let rows = if let Some(ref mt) = q.media_type {
        state
            .recommendations
            .top_recommended_by_source_type(mt, limit)
    } else {
        state.recommendations.top_recommended_movies(limit)
    };
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|(tmdb_id, media_type, title, count)| {
            serde_json::json!({
                "tmdbId": tmdb_id,
                "mediaType": media_type,
                "title": title,
                "count": count,
            })
        })
        .collect();
    Json(serde_json::json!(result))
}

async fn top_movies_detail(
    State(state): State<AppState>,
    Query(q): Query<FilterQuery>,
) -> impl IntoResponse {
    let limit = q.limit.unwrap_or(50);
    let rows = state.recommendations.top_recommended_movies_with_data(limit);
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|(tmdb_id, media_type, title, count, data)| {
            serde_json::json!({
                "tmdbId": tmdb_id,
                "mediaType": media_type,
                "title": title,
                "count": count,
                "data": data,
            })
        })
        .collect();
    Json(serde_json::json!(result))
}

async fn top_genres(
    State(state): State<AppState>,
    Query(q): Query<FilterQuery>,
) -> impl IntoResponse {
    let limit = q.limit.unwrap_or(50);
    let rows = state.recommendations.top_genres(limit);
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|(genre, count)| {
            serde_json::json!({
                "genre": genre,
                "count": count,
            })
        })
        .collect();
    Json(serde_json::json!(result))
}
