use crate::AppState;
use std::collections::HashSet;
use std::time::Duration;
use tracing::{error, info, warn};

const THROTTLE_DELAY: Duration = Duration::from_secs(1);
const OL_BASE: &str = "https://openlibrary.org";
const MAX_SUBJECTS: usize = 10;
const SUBJECT_LIMIT: usize = 50;

fn max_level() -> i64 {
    std::env::var("BOOK_RECOMMENDATIONS_MAX_LEVEL")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2)
}

pub async fn run_book_recommendations_worker(state: AppState) {
    info!("[book-recs-worker] Starting book recommendations queue worker");

    let client = reqwest::Client::builder()
        .user_agent("mhaol/1.0.0 (https://github.com/arktosmos/mhaol)")
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");

    loop {
        let task = state
            .queue
            .claim_next(mhaol_recommendations::books::BOOK_TASK_PREFIX);

        match task {
            Some(task) => {
                info!(
                    "[book-recs-worker] Processing task {} ({})",
                    task.id, task.task_type
                );
                match task.task_type.as_str() {
                    mhaol_recommendations::books::BOOK_TASK_FETCH => {
                        process_fetch_related_books(&state, &client, &task).await;
                    }
                    other => {
                        warn!("[book-recs-worker] Unknown task type: {}", other);
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

async fn process_fetch_related_books(
    state: &AppState,
    client: &reqwest::Client,
    task: &mhaol_queue::QueueTask,
) {
    let payload = &task.payload;

    let key = match payload["key"].as_str() {
        Some(k) if !k.is_empty() => k,
        _ => {
            state
                .queue
                .fail(&task.id, "Missing or invalid key in payload");
            return;
        }
    };

    let level = payload["level"].as_i64().unwrap_or(1);

    // Fetch work details to get subjects
    let work_url = format!("{}/works/{}.json", OL_BASE, key);
    let work_data = match client.get(&work_url).send().await {
        Ok(resp) if resp.status().is_success() => match resp.json::<serde_json::Value>().await {
            Ok(d) => d,
            Err(e) => {
                error!("[book-recs-worker] Failed to parse work response: {}", e);
                state.queue.fail(&task.id, &format!("Parse error: {}", e));
                return;
            }
        },
        Ok(resp) => {
            let status = resp.status();
            state
                .queue
                .fail(&task.id, &format!("Work fetch failed: HTTP {}", status));
            return;
        }
        Err(e) => {
            error!("[book-recs-worker] Failed to fetch work: {}", e);
            state.queue.fail(&task.id, &e.to_string());
            return;
        }
    };

    let source_title = work_data["title"].as_str().unwrap_or("");

    // Extract subjects array
    let subjects: Vec<String> = work_data["subjects"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    if subjects.is_empty() {
        info!(
            "[book-recs-worker] No subjects for work {} — skipping",
            key
        );
        state.queue.complete(
            &task.id,
            serde_json::json!({
                "key": key,
                "title": source_title,
                "count": 0,
                "level": level,
            }),
        );
        return;
    }

    // Pick top subjects (more specific ones tend to be later in the list, but take first N for breadth)
    let selected_subjects: Vec<&str> = subjects
        .iter()
        .map(|s| s.as_str())
        .take(MAX_SUBJECTS)
        .collect();

    let mut inserted = 0;
    let mut rec_keys: Vec<String> = Vec::new();
    let mut score_map: std::collections::HashMap<String, f64> = std::collections::HashMap::new();

    for subject in &selected_subjects {
        // Normalize subject for URL: lowercase, replace spaces with underscores
        let normalized = subject.to_lowercase().replace(' ', "_");
        let url = format!(
            "{}/subjects/{}.json?limit={}",
            OL_BASE, normalized, SUBJECT_LIMIT
        );

        tokio::time::sleep(THROTTLE_DELAY).await;

        let resp = match client.get(&url).send().await {
            Ok(r) if r.status().is_success() => match r.json::<serde_json::Value>().await {
                Ok(d) => d,
                Err(_) => continue,
            },
            _ => continue,
        };

        let works = match resp["works"].as_array() {
            Some(arr) => arr,
            None => continue,
        };

        for work in works {
            let work_key = match work["key"].as_str() {
                Some(k) => {
                    // Keys come as "/works/OL123W" — strip the prefix
                    k.strip_prefix("/works/").unwrap_or(k)
                }
                None => continue,
            };

            // Skip self-references
            if work_key == key {
                continue;
            }

            // Increment score for this recommendation (more shared subjects = higher score)
            let entry = score_map.entry(work_key.to_string()).or_insert(0.0);
            *entry += 1.0;

            let title = work["title"].as_str();
            let authors = work["authors"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|a| a["name"].as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .or_else(|| {
                    work["author_name"]
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|a| a.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                });

            let work_subjects = work["subject"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str())
                        .take(10)
                        .collect::<Vec<_>>()
                        .join(", ")
                });

            let data_str = serde_json::to_string(work).unwrap_or_default();

            state.book_recommendations.upsert(
                key,
                work_key,
                title,
                authors.as_deref(),
                work_subjects.as_deref(),
                *entry,
                level,
                &data_str,
            );

            if !rec_keys.contains(&work_key.to_string()) {
                rec_keys.push(work_key.to_string());
            }
            inserted += 1;
        }
    }

    // Auto-enqueue next level
    let max = max_level();
    if level < max && !rec_keys.is_empty() {
        let next_level = level + 1;
        let existing_tasks = state
            .queue
            .list(None, Some(mhaol_recommendations::books::BOOK_TASK_FETCH));
        let queued_keys: HashSet<String> = existing_tasks
            .iter()
            .filter(|t| {
                t.status == mhaol_queue::QueueTaskStatus::Pending
                    || t.status == mhaol_queue::QueueTaskStatus::Running
            })
            .filter_map(|t| t.payload["key"].as_str().map(|s| s.to_string()))
            .collect();

        let mut enqueued = 0;
        for rec_key in &rec_keys {
            if state.book_recommendations.has_source(rec_key) {
                continue;
            }
            if queued_keys.contains(rec_key) {
                continue;
            }
            state.queue.enqueue(
                mhaol_recommendations::books::BOOK_TASK_FETCH,
                serde_json::json!({
                    "key": rec_key,
                    "level": next_level,
                }),
            );
            enqueued += 1;
        }
        if enqueued > 0 {
            info!(
                "[book-recs-worker] Auto-enqueued {} level {} tasks from book {}",
                enqueued, next_level, key
            );
        }
    }

    info!(
        "[book-recs-worker] Task {} completed: {} related books for {} (level {})",
        task.id, inserted, key, level
    );
    state.queue.complete(
        &task.id,
        serde_json::json!({
            "key": key,
            "title": source_title,
            "count": inserted,
            "level": level,
        }),
    );
}
