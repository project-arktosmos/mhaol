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
                "level": 1,
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
    let rows = state
        .recommendations
        .top_recommended_with_level_counts(q.media_type.as_deref(), limit);

    // Compute totals per level across all results
    let mut level_totals: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
    for (_, _, _, _, ref level_counts) in &rows {
        for (&lvl, &cnt) in level_counts {
            *level_totals.entry(lvl).or_insert(0) += cnt;
        }
    }

    // Collect all levels sorted
    let mut levels: Vec<i64> = level_totals.keys().copied().collect();
    levels.sort();

    // Build result with percentages and scores, sorted by score desc
    let mut result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|(tmdb_id, media_type, title, count, level_counts)| {
            let mut lc = serde_json::Map::new();
            let mut lp = serde_json::Map::new();
            let mut score: f64 = 0.0;

            for &lvl in &levels {
                let cnt = level_counts.get(&lvl).copied().unwrap_or(0);
                let total = level_totals.get(&lvl).copied().unwrap_or(0);
                let pct = if total > 0 {
                    ((cnt as f64 / total as f64) * 100.0).round() as i64
                } else {
                    0
                };
                let divisor = 1.0 + (lvl as f64 / 10.0);
                score += pct as f64 / divisor;
                lc.insert(lvl.to_string(), serde_json::Value::from(cnt));
                lp.insert(lvl.to_string(), serde_json::Value::from(pct));
            }

            let rounded_score = (score * 10.0).round() / 10.0;
            serde_json::json!({
                "tmdbId": tmdb_id,
                "mediaType": media_type,
                "title": title,
                "count": count,
                "levelCounts": lc,
                "levelPercentages": lp,
                "score": rounded_score,
                "levels": levels,
            })
        })
        .collect();

    result.sort_by(|a, b| {
        let sa = a["score"].as_f64().unwrap_or(0.0);
        let sb = b["score"].as_f64().unwrap_or(0.0);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });

    Json(serde_json::json!(result))
}

async fn top_movies_detail(
    State(state): State<AppState>,
    Query(q): Query<FilterQuery>,
) -> impl IntoResponse {
    let limit = q.limit.unwrap_or(50);
    let rows = state.recommendations.top_recommended_movies_with_data(limit);

    // Collect recommended IDs to batch-fetch their sources
    let rec_ids: Vec<i64> = rows.iter().map(|(id, _, _, _, _, _)| *id).collect();
    let source_rows = state.recommendations.sources_for_recommended(&rec_ids);

    // Group sources by recommended_tmdb_id
    let mut sources_map: std::collections::HashMap<i64, Vec<serde_json::Value>> =
        std::collections::HashMap::new();
    for (rec_id, src_id, src_media_type, src_title) in source_rows {
        sources_map.entry(rec_id).or_default().push(serde_json::json!({
            "tmdbId": src_id,
            "mediaType": src_media_type,
            "title": src_title,
        }));
    }

    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|(tmdb_id, media_type, title, count, min_level, data)| {
            let sources = sources_map
                .remove(&tmdb_id)
                .unwrap_or_default();
            serde_json::json!({
                "tmdbId": tmdb_id,
                "mediaType": media_type,
                "title": title,
                "count": count,
                "minLevel": min_level,
                "data": data,
                "sources": sources,
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
