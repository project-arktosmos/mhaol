use crate::AppState;
use std::collections::HashSet;
use std::time::Duration;
use tracing::{error, info, warn};

const THROTTLE_DELAY: Duration = Duration::from_secs(1);
const LISTENBRAINZ_LABS_URL: &str = "https://labs.api.listenbrainz.org/similar-artists/json";
const DEFAULT_ALGORITHM: &str =
    "session_based_days_7500_session_300_contribution_5_threshold_10_limit_100_filter_True_skip_30";

fn max_level() -> i64 {
    std::env::var("MUSIC_RECOMMENDATIONS_MAX_LEVEL")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2)
}

pub async fn run_music_recommendations_worker(state: AppState) {
    info!("[music-recs-worker] Starting music recommendations queue worker");

    let client = reqwest::Client::builder()
        .user_agent("mhaol/1.0.0 (https://github.com/arktosmos/mhaol)")
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");

    loop {
        let task = state
            .queue
            .claim_next(mhaol_recommendations::music::MUSIC_TASK_PREFIX);

        match task {
            Some(task) => {
                info!(
                    "[music-recs-worker] Processing task {} ({})",
                    task.id, task.task_type
                );
                match task.task_type.as_str() {
                    mhaol_recommendations::music::MUSIC_TASK_FETCH => {
                        process_fetch_similar_artists(&state, &client, &task).await;
                    }
                    other => {
                        warn!("[music-recs-worker] Unknown task type: {}", other);
                        state
                            .queue
                            .fail(&task.id, &format!("Unknown task type: {}", other));
                    }
                }
                tokio::time::sleep(THROTTLE_DELAY).await;
            }
            None => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn process_fetch_similar_artists(
    state: &AppState,
    client: &reqwest::Client,
    task: &mhaol_queue::QueueTask,
) {
    let payload = &task.payload;

    let mbid = match payload["mbid"].as_str() {
        Some(id) if id.len() == 36 => id,
        _ => {
            state
                .queue
                .fail(&task.id, "Missing or invalid mbid in payload");
            return;
        }
    };

    let level = payload["level"].as_i64().unwrap_or(1);

    let request_body = serde_json::json!([{
        "artist_mbids": [mbid],
        "algorithm": DEFAULT_ALGORITHM,
    }]);

    let response = client
        .post(LISTENBRAINZ_LABS_URL)
        .json(&request_body)
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let data: serde_json::Value = match resp.json().await {
                Ok(d) => d,
                Err(e) => {
                    error!("[music-recs-worker] Failed to parse response: {}", e);
                    state.queue.fail(&task.id, &format!("Parse error: {}", e));
                    return;
                }
            };

            let arr = data.as_array();
            let mut inserted = 0;
            let mut rec_mbids: Vec<String> = Vec::new();

            if let Some(arr) = arr {
                for item in arr {
                    let rec_mbid = match item["artist_mbid"].as_str() {
                        Some(id) => id,
                        None => continue,
                    };

                    // Skip self-references
                    if rec_mbid == mbid {
                        continue;
                    }

                    let name = item["name"].as_str();
                    let score = item["score"].as_f64().or_else(|| item["score"].as_i64().map(|i| i as f64));
                    let artist_type = item["type"].as_str().unwrap_or("");
                    let comment = item["comment"].as_str().unwrap_or("");
                    let gender = item["gender"].as_str();

                    let data_str = serde_json::to_string(item).unwrap_or_default();

                    // Build tags from type, comment, gender
                    let mut tags_parts: Vec<&str> = Vec::new();
                    if !artist_type.is_empty() {
                        tags_parts.push(artist_type);
                    }
                    if let Some(g) = gender {
                        tags_parts.push(g);
                    }
                    if !comment.is_empty() {
                        tags_parts.push(comment);
                    }
                    let tags = if tags_parts.is_empty() {
                        None
                    } else {
                        Some(tags_parts.join(", "))
                    };

                    state.music_recommendations.upsert(
                        mbid,
                        "artist",
                        rec_mbid,
                        "artist",
                        name,
                        tags.as_deref(),
                        score,
                        level,
                        &data_str,
                    );
                    rec_mbids.push(rec_mbid.to_string());
                    inserted += 1;
                }
            }

            // Auto-enqueue next level
            let max = max_level();
            if level < max && !rec_mbids.is_empty() {
                let next_level = level + 1;
                let existing_tasks = state
                    .queue
                    .list(None, Some(mhaol_recommendations::music::MUSIC_TASK_FETCH));
                let queued_mbids: HashSet<String> = existing_tasks
                    .iter()
                    .filter(|t| {
                        t.status == mhaol_queue::QueueTaskStatus::Pending
                            || t.status == mhaol_queue::QueueTaskStatus::Running
                    })
                    .filter_map(|t| t.payload["mbid"].as_str().map(|s| s.to_string()))
                    .collect();

                let mut enqueued = 0;
                for rec_mbid in &rec_mbids {
                    if state.music_recommendations.has_source(rec_mbid, "artist") {
                        continue;
                    }
                    if queued_mbids.contains(rec_mbid) {
                        continue;
                    }
                    state.queue.enqueue(
                        mhaol_recommendations::music::MUSIC_TASK_FETCH,
                        serde_json::json!({
                            "mbid": rec_mbid,
                            "level": next_level,
                        }),
                    );
                    enqueued += 1;
                }
                if enqueued > 0 {
                    info!(
                        "[music-recs-worker] Auto-enqueued {} level {} tasks from artist {}",
                        enqueued, next_level, mbid
                    );
                }
            }

            info!(
                "[music-recs-worker] Task {} completed: {} similar artists for {} (level {})",
                task.id, inserted, mbid, level
            );
            state.queue.complete(
                &task.id,
                serde_json::json!({
                    "mbid": mbid,
                    "count": inserted,
                    "level": level,
                }),
            );
        }
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!(
                "[music-recs-worker] Task {} failed: HTTP {} — {}",
                task.id, status, body
            );
            state
                .queue
                .fail(&task.id, &format!("HTTP {}: {}", status, body));
        }
        Err(e) => {
            error!("[music-recs-worker] Task {} failed: {}", task.id, e);
            state.queue.fail(&task.id, &e.to_string());
        }
    }
}
