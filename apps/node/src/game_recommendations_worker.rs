use crate::AppState;
use std::time::Duration;
use tracing::{info, warn};

pub async fn run_game_recommendations_worker(state: AppState) {
    info!("[game-recs-worker] Starting game recommendations queue worker");

    loop {
        let task = state
            .queue
            .claim_next(mhaol_recommendations::game::GAME_TASK_PREFIX);

        match task {
            Some(task) => {
                info!(
                    "[game-recs-worker] Processing task {} ({})",
                    task.id, task.task_type
                );
                match task.task_type.as_str() {
                    mhaol_recommendations::game::GAME_TASK_FETCH => {
                        process_fetch_game_recommendations(&state, &task).await;
                    }
                    other => {
                        warn!("[game-recs-worker] Unknown task type: {}", other);
                        state
                            .queue
                            .fail(&task.id, &format!("Unknown task type: {}", other));
                    }
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            None => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn process_fetch_game_recommendations(state: &AppState, task: &mhaol_queue::QueueTask) {
    let payload = &task.payload;

    let game_id = match payload["gameId"].as_i64() {
        Some(id) => id,
        None => {
            state.queue.fail(&task.id, "Missing gameId in payload");
            return;
        }
    };

    let level = payload["level"].as_i64().unwrap_or(1);

    // Fetch source game details from cache
    let source_detail = state
        .api_cache
        .get_any("retroachievements", &format!("game-details:{}", game_id))
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());

    let source_detail = match source_detail {
        Some(d) => d,
        None => {
            state
                .queue
                .fail(&task.id, "Source game not found in cache — browse the game first to populate cache");
            return;
        }
    };

    let source_genre = source_detail["Genre"]
        .as_str()
        .unwrap_or("")
        .to_lowercase();
    let source_developer = source_detail["Developer"]
        .as_str()
        .unwrap_or("")
        .to_lowercase();
    let source_publisher = source_detail["Publisher"]
        .as_str()
        .unwrap_or("")
        .to_lowercase();
    let source_console_id = source_detail["ConsoleID"]
        .as_i64()
        .or_else(|| source_detail["ConsoleID"].as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(0);

    if source_genre.is_empty() && source_developer.is_empty() && source_publisher.is_empty() {
        state.queue.complete(
            &task.id,
            serde_json::json!({ "gameId": game_id, "count": 0, "level": level, "reason": "no metadata" }),
        );
        return;
    }

    let source_genres: Vec<String> = source_genre
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Load all cached game lists
    let game_lists: Vec<(i64, serde_json::Value)> = state
        .api_cache
        .get_all_for_prefix("retroachievements", "game-list:")
        .into_iter()
        .filter_map(|(key, data_str)| {
            let cid = key
                .strip_prefix("game-list:")
                .and_then(|s| s.parse::<i64>().ok())?;
            let v = serde_json::from_str::<serde_json::Value>(&data_str).ok()?;
            Some((cid, v))
        })
        .collect();

    // Collect all candidate games from game lists
    struct Candidate {
        id: i64,
        title: String,
        console_id: i64,
        console_name: String,
        num_achievements: i64,
        data: serde_json::Value,
    }

    let mut candidates: Vec<Candidate> = Vec::new();
    for (console_id, list_data) in &game_lists {
        if let Some(arr) = list_data.as_array() {
            for item in arr {
                let id = item["ID"]
                    .as_i64()
                    .or_else(|| item["ID"].as_str().and_then(|s| s.parse().ok()))
                    .unwrap_or(0);
                if id == 0 || id == game_id {
                    continue;
                }
                let title = item["Title"].as_str().unwrap_or("").to_string();
                let console_name = item["ConsoleName"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                let num_achievements = item["NumAchievements"]
                    .as_i64()
                    .or_else(|| item["NumAchievements"].as_str().and_then(|s| s.parse().ok()))
                    .unwrap_or(0);
                candidates.push(Candidate {
                    id,
                    title,
                    console_id: *console_id,
                    console_name,
                    num_achievements,
                    data: item.clone(),
                });
            }
        }
    }

    // Score candidates using cached detail data where available
    struct ScoredCandidate {
        id: i64,
        title: String,
        genre: Option<String>,
        console_id: i64,
        console_name: String,
        score: f64,
        data: serde_json::Value,
    }

    let mut scored: Vec<ScoredCandidate> = Vec::new();

    for candidate in &candidates {
        let mut match_score: f64 = 0.0;

        // Try to get detail from cache for genre/developer/publisher matching
        let detail = state
            .api_cache
            .get_any("retroachievements", &format!("game-details:{}", candidate.id))
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok());

        let candidate_genre;
        let candidate_developer;
        let candidate_publisher;

        if let Some(ref detail) = detail {
            candidate_genre = detail["Genre"]
                .as_str()
                .unwrap_or("")
                .to_lowercase();
            candidate_developer = detail["Developer"]
                .as_str()
                .unwrap_or("")
                .to_lowercase();
            candidate_publisher = detail["Publisher"]
                .as_str()
                .unwrap_or("")
                .to_lowercase();
        } else {
            candidate_genre = String::new();
            candidate_developer = String::new();
            candidate_publisher = String::new();
        }

        // Genre matching
        if !candidate_genre.is_empty() && !source_genres.is_empty() {
            let candidate_genres: Vec<String> = candidate_genre
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let mut genre_matches = 0;
            for sg in &source_genres {
                for cg in &candidate_genres {
                    if sg == cg {
                        genre_matches += 1;
                    }
                }
            }
            if genre_matches > 0 {
                match_score += 50.0 * genre_matches as f64;
            }
        }

        // Developer match
        if !source_developer.is_empty()
            && !candidate_developer.is_empty()
            && source_developer == candidate_developer
        {
            match_score += 30.0;
        }

        // Publisher match
        if !source_publisher.is_empty()
            && !candidate_publisher.is_empty()
            && source_publisher == candidate_publisher
        {
            match_score += 20.0;
        }

        // Same console bonus
        if candidate.console_id == source_console_id {
            match_score += 10.0;
        }

        // Has achievements bonus
        if candidate.num_achievements > 0 {
            match_score += 5.0;
        }

        if match_score >= 50.0 {
            scored.push(ScoredCandidate {
                id: candidate.id,
                title: candidate.title.clone(),
                genre: if candidate_genre.is_empty() {
                    None
                } else {
                    Some(candidate_genre)
                },
                console_id: candidate.console_id,
                console_name: candidate.console_name.clone(),
                score: match_score,
                data: candidate.data.clone(),
            });
        }
    }

    // Sort by score descending, take top 100
    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(100);

    let mut inserted = 0;
    for sc in &scored {
        let data_str = serde_json::to_string(&sc.data).unwrap_or_default();
        state.game_recommendations.upsert(
            game_id,
            sc.id,
            Some(&sc.title),
            sc.genre.as_deref(),
            Some(sc.console_id),
            Some(&sc.console_name),
            sc.score,
            level,
            &data_str,
        );
        inserted += 1;
    }

    info!(
        "[game-recs-worker] Task {} completed: {} recommendations for game {} (level {})",
        task.id, inserted, game_id, level
    );
    state.queue.complete(
        &task.id,
        serde_json::json!({
            "gameId": game_id,
            "count": inserted,
            "level": level,
        }),
    );
}
