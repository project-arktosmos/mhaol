use crate::api::tmdb::tmdb_fetch_json;
use crate::AppState;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tracing::{error, info, warn};

const THROTTLE_DELAY: Duration = Duration::from_secs(2);

fn max_recommendation_level() -> i64 {
    std::env::var("RECOMMENDATIONS_MAX_LEVEL")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3)
}

pub async fn run_recommendations_worker(state: AppState) {
    info!("[recommendations-worker] Starting recommendations queue worker");

    loop {
        let task = state.queue.claim_next(mhaol_recommendations::TASK_PREFIX);

        match task {
            Some(task) => {
                info!(
                    "[recommendations-worker] Processing task {} ({})",
                    task.id, task.task_type
                );
                match task.task_type.as_str() {
                    mhaol_recommendations::TASK_FETCH => {
                        process_fetch_recommendations(&state, &task).await;
                    }
                    other => {
                        warn!("[recommendations-worker] Unknown task type: {}", other);
                        state
                            .queue
                            .fail(&task.id, &format!("Unknown task type: {}", other));
                    }
                }
                // Throttle: cool off between TMDB API calls
                tokio::time::sleep(THROTTLE_DELAY).await;
            }
            None => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn process_fetch_recommendations(state: &AppState, task: &mhaol_queue::QueueTask) {
    let payload = &task.payload;

    let tmdb_id = match payload["tmdbId"].as_i64() {
        Some(id) => id,
        None => {
            state.queue.fail(&task.id, "Missing tmdbId in payload");
            return;
        }
    };

    let media_type = match payload["mediaType"].as_str() {
        Some(mt) if mt == "movie" || mt == "tv" => mt,
        _ => {
            state.queue.fail(
                &task.id,
                "Missing or invalid mediaType in payload (must be 'movie' or 'tv')",
            );
            return;
        }
    };

    let level = payload["level"].as_i64().unwrap_or(1);

    // Fetch genre list for name resolution
    let genre_path = format!("/genre/{}/list", media_type);
    let genre_map: HashMap<i64, String> =
        match tmdb_fetch_json(state, &genre_path, &[]).await {
            Ok(data) => data["genres"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|g| {
                            let id = g["id"].as_i64()?;
                            let name = g["name"].as_str()?;
                            Some((id, name.to_string()))
                        })
                        .collect()
                })
                .unwrap_or_default(),
            Err(_) => HashMap::new(),
        };

    let path = format!("/{}/{}/recommendations", media_type, tmdb_id);
    match tmdb_fetch_json(state, &path, &[("page", "1")]).await {
        Ok(data) => {
            let arr = data["results"].as_array();
            let mut inserted = 0;
            let mut rec_ids: Vec<i64> = Vec::new();

            if let Some(arr) = arr {
                for item in arr {
                    let rec_id = item["id"].as_i64().unwrap_or(0);
                    if rec_id == 0 {
                        continue;
                    }
                    let title = item["title"].as_str().or_else(|| item["name"].as_str());
                    let genres = item["genre_ids"]
                        .as_array()
                        .map(|ids| {
                            ids.iter()
                                .filter_map(|id| {
                                    id.as_i64().and_then(|n| genre_map.get(&n).cloned())
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .filter(|s| !s.is_empty());
                    let data_str = serde_json::to_string(item).unwrap_or_default();
                    state.recommendations.upsert(
                        tmdb_id,
                        media_type,
                        rec_id,
                        media_type,
                        title,
                        genres.as_deref(),
                        level,
                        &data_str,
                    );
                    rec_ids.push(rec_id);
                    inserted += 1;
                }
            }

            // Auto-enqueue next level recommendations
            let max_level = max_recommendation_level();
            if level < max_level && !rec_ids.is_empty() {
                let next_level = level + 1;
                let existing_tasks = state
                    .queue
                    .list(None, Some(mhaol_recommendations::TASK_FETCH));
                let queued_ids: HashSet<(i64, String)> = existing_tasks
                    .iter()
                    .filter(|t| {
                        t.status == mhaol_queue::QueueTaskStatus::Pending
                            || t.status == mhaol_queue::QueueTaskStatus::Running
                    })
                    .filter_map(|t| {
                        let id = t.payload["tmdbId"].as_i64()?;
                        let mt = t.payload["mediaType"].as_str()?.to_string();
                        Some((id, mt))
                    })
                    .collect();

                let mut enqueued = 0;
                for rec_id in &rec_ids {
                    if state.recommendations.has_source(*rec_id, media_type) {
                        continue;
                    }
                    if queued_ids.contains(&(*rec_id, media_type.to_string())) {
                        continue;
                    }
                    state.queue.enqueue(
                        mhaol_recommendations::TASK_FETCH,
                        serde_json::json!({
                            "tmdbId": rec_id,
                            "mediaType": media_type,
                            "level": next_level,
                        }),
                    );
                    enqueued += 1;
                }
                if enqueued > 0 {
                    info!(
                        "[recommendations-worker] Auto-enqueued {} level {} tasks from {} {}",
                        enqueued, next_level, media_type, tmdb_id
                    );
                }
            }

            info!(
                "[recommendations-worker] Task {} completed: {} recommendations for {} {} (level {})",
                task.id, inserted, media_type, tmdb_id, level
            );
            state.queue.complete(
                &task.id,
                serde_json::json!({
                    "tmdbId": tmdb_id,
                    "mediaType": media_type,
                    "count": inserted,
                    "level": level,
                }),
            );
        }
        Err(e) => {
            error!("[recommendations-worker] Task {} failed: {}", task.id, e);
            state.queue.fail(&task.id, &e);
        }
    }
}
