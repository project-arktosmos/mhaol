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
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/bulk", post(bulk_enqueue))
        .route("/status", get(status))
        .route("/top", get(top_books))
        .route("/top-detail", get(top_books_detail))
        .route(
            "/labels",
            get(get_label_assignments)
                .put(set_label)
                .delete(remove_label),
        )
        .route("/{key}", get(get_recommendations))
}

#[derive(Deserialize)]
struct BulkItem {
    key: String,
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
        .list(None, Some(mhaol_recommendations::books::BOOK_TASK_FETCH));

    let mut enqueued = 0;
    for item in &body.items {
        if item.key.is_empty() {
            continue;
        }

        // Cancel/remove existing queue tasks for this key
        for task in &existing_tasks {
            if let Some(k) = task.payload.get("key").and_then(|v| v.as_str()) {
                if k == item.key {
                    let _ = state.queue.cancel(&task.id);
                    state.queue.remove(&task.id);
                }
            }
        }

        // Delete existing recommendation records
        state.book_recommendations.delete_for_source(&item.key);

        state.queue.enqueue(
            mhaol_recommendations::books::BOOK_TASK_FETCH,
            serde_json::json!({
                "key": item.key,
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
    Path(key): Path<String>,
) -> impl IntoResponse {
    let recs = state.book_recommendations.get_for_source(&key);
    Json(serde_json::json!(recs))
}

async fn status(State(state): State<AppState>) -> impl IntoResponse {
    let all = state
        .queue
        .list(None, Some(mhaol_recommendations::books::BOOK_TASK_FETCH));
    let pending = all
        .iter()
        .filter(|t| t.status == mhaol_queue::QueueTaskStatus::Pending)
        .count();
    let running = all
        .iter()
        .filter(|t| t.status == mhaol_queue::QueueTaskStatus::Running)
        .count();
    let completed = all
        .iter()
        .filter(|t| t.status == mhaol_queue::QueueTaskStatus::Completed)
        .count();
    let failed = all
        .iter()
        .filter(|t| t.status == mhaol_queue::QueueTaskStatus::Failed)
        .count();
    Json(serde_json::json!({
        "pending": pending,
        "running": running,
        "completed": completed,
        "failed": failed,
        "total": all.len(),
    }))
}

async fn top_books(
    State(state): State<AppState>,
    Query(q): Query<FilterQuery>,
) -> impl IntoResponse {
    let limit = q.limit.unwrap_or(50);
    let rows = state
        .book_recommendations
        .top_recommended_with_level_counts(limit);

    // Compute totals per level across all results
    let mut level_totals: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
    for (_, _, _, ref level_counts) in &rows {
        for (&lvl, &cnt) in level_counts {
            *level_totals.entry(lvl).or_insert(0) += cnt;
        }
    }

    let mut levels: Vec<i64> = level_totals.keys().copied().collect();
    levels.sort();

    let mut result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|(key, title, count, level_counts)| {
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
                "key": key,
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

async fn top_books_detail(
    State(state): State<AppState>,
    Query(q): Query<FilterQuery>,
) -> impl IntoResponse {
    let limit = q.limit.unwrap_or(50);
    let rows = state.book_recommendations.top_recommended_with_data(limit);

    let rec_keys: Vec<String> = rows.iter().map(|(key, _, _, _, _)| key.clone()).collect();
    let source_rows = state
        .book_recommendations
        .sources_for_recommended(&rec_keys);

    let mut sources_map: std::collections::HashMap<String, Vec<serde_json::Value>> =
        std::collections::HashMap::new();
    for (rec_key, src_key, src_title) in source_rows {
        sources_map
            .entry(rec_key)
            .or_default()
            .push(serde_json::json!({
                "key": src_key,
                "title": src_title,
            }));
    }

    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|(key, title, count, min_level, data)| {
            let sources = sources_map.remove(&key).unwrap_or_default();
            serde_json::json!({
                "key": key,
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

// --- Book recommendation label endpoints ---

#[derive(Deserialize)]
struct WalletQuery {
    wallet: String,
}

async fn get_label_assignments(
    State(state): State<AppState>,
    Query(q): Query<WalletQuery>,
) -> impl IntoResponse {
    let rows = state.recommendation_labels.get_assignments_by_wallet_and_source(&q.wallet, "openlibrary");
    let mapped: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
        "id": r.id,
        "wallet": r.wallet,
        "recommendedKey": r.source_id,
        "labelId": r.label_id,
        "createdAt": r.created_at,
    })).collect();
    Json(serde_json::json!(mapped))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetLabelBody {
    wallet: String,
    recommended_key: String,
    label_id: String,
}

async fn set_label(
    State(state): State<AppState>,
    Json(body): Json<SetLabelBody>,
) -> impl IntoResponse {
    let ok = state.recommendation_labels.upsert(&body.wallet, "openlibrary", &body.recommended_key, "book", &body.label_id);
    if ok {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoveLabelBody {
    wallet: String,
    recommended_key: String,
}

async fn remove_label(
    State(state): State<AppState>,
    Json(body): Json<RemoveLabelBody>,
) -> impl IntoResponse {
    let deleted = state.recommendation_labels.delete(&body.wallet, "openlibrary", &body.recommended_key, "book");
    if deleted {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}
