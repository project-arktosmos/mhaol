use crate::AppState;
use mhaol_llm::{build_chat_prompt, list_models, load_model_blocking, run_inference_blocking, ChatMessage, LlmTokenEvent};
use tracing::{info, warn};

/// Try to load the first available .gguf model if none is loaded.
async fn ensure_model_loaded(state: &AppState) {
    if state.llm_engine.is_model_loaded() {
        return;
    }

    let models = list_models(&state.llm_engine);
    let first = match models.first() {
        Some(m) => m,
        None => return,
    };

    let model_path = state.llm_engine.models_dir.join(&first.file_name);
    let file_name = first.file_name.clone();
    info!("[llm-worker] Auto-loading model: {}", file_name);

    match tokio::task::spawn_blocking(move || load_model_blocking(model_path)).await {
        Ok(Ok(model)) => {
            state.llm_engine.set_model(std::sync::Arc::new(model), file_name.clone());
            info!("[llm-worker] Model {} loaded", file_name);
        }
        Ok(Err(e)) => warn!("[llm-worker] Failed to load model: {}", e),
        Err(e) => warn!("[llm-worker] Model load task panicked: {:?}", e),
    }
}

pub async fn run_llm_worker(state: AppState) {
    info!("[llm-worker] Starting LLM queue worker");

    // Load model at startup
    ensure_model_loaded(&state).await;

    loop {
        let task = state.queue.claim_next("llm:");

        match task {
            Some(task) => {
                // Try to load a model if one appeared since last check (e.g. user downloaded one)
                ensure_model_loaded(&state).await;

                info!("[llm-worker] Processing task {} ({})", task.id, task.task_type);
                match task.task_type.as_str() {
                    "llm:analyze-torrent" => process_analyze_torrent(&state, &task).await,
                    other => {
                        warn!("[llm-worker] Unknown task type: {}", other);
                        state.queue.fail(&task.id, &format!("Unknown task type: {}", other));
                    }
                }
            }
            None => {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
}

async fn process_analyze_torrent(state: &AppState, task: &mhaol_queue::QueueTask) {
    let payload = &task.payload;

    let torrent_name = payload["torrentName"].as_str().unwrap_or("");
    let media_title = payload["mediaTitle"].as_str().unwrap_or("");
    let media_year = payload["mediaYear"].as_u64().unwrap_or(0);
    let artist = payload["artist"].as_str();
    let console_name = payload["consoleName"].as_str();
    let prompt_template = payload["promptTemplate"].as_str().unwrap_or("");

    if torrent_name.is_empty() || media_title.is_empty() {
        state.queue.fail(&task.id, "Missing torrentName or mediaTitle in payload");
        return;
    }

    if !state.llm_engine.is_model_loaded() {
        state.queue.fail(&task.id, "No LLM model available");
        return;
    }

    // Build prompt from template or use default
    let prompt_text = if prompt_template.is_empty() {
        format!(
            "Given a torrent name and a target media, determine whether the torrent matches. \
             Respond in JSON only.\n\n\
             Target: {} ({})\nTorrent: {}\n\n\
             Respond with:\n\
             {{\"quality\": \"<quality>\", \"languages\": \"<languages>\", \"subs\": \"<subs or none>\", \
             \"relevance\": <0-100>, \"reason\": \"<brief explanation>\"}}",
            media_title, media_year, torrent_name
        )
    } else {
        prompt_template
            .replace("{{title}}", media_title)
            .replace("{{year}}", &media_year.to_string())
            .replace("{{torrentName}}", torrent_name)
            .replace("{{artist}}", artist.unwrap_or(""))
            .replace("{{console}}", console_name.unwrap_or(""))
    };

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: prompt_text,
    }];

    let config = state.llm_engine.get_config();
    let prompt = build_chat_prompt(&messages, &config.system_prompt);
    let max_tokens = config.max_tokens.min(512);

    let model = match state.llm_engine.get_model() {
        Ok(m) => m,
        Err(e) => {
            state.queue.fail(&task.id, &format!("Failed to get model: {}", e));
            return;
        }
    };

    state.llm_engine.reset_cancel_flag();
    let cancel_flag = state.llm_engine.cancel_flag();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<LlmTokenEvent>(64);

    tokio::task::spawn_blocking(move || {
        run_inference_blocking(model, prompt, max_tokens, cancel_flag, tx);
    });

    let mut full_response = String::new();
    while let Some(event) = rx.recv().await {
        if event.done {
            break;
        }
        full_response.push_str(&event.content);
    }

    let full_response = full_response.trim().to_string();

    if full_response.is_empty() {
        state.queue.fail(&task.id, "LLM returned empty response");
        return;
    }

    let json_result = extract_json(&full_response);

    match json_result {
        Some(parsed) => {
            info!("[llm-worker] Task {} completed", task.id);
            state.queue.complete(&task.id, parsed);
        }
        None => {
            warn!("[llm-worker] Task {} — could not parse JSON from LLM output", task.id);
            state.queue.complete(
                &task.id,
                serde_json::json!({
                    "raw": full_response,
                    "relevance": 0,
                    "quality": "",
                    "languages": "",
                    "subs": "",
                    "reason": "Failed to parse LLM output"
                }),
            );
        }
    }
}

fn extract_json(text: &str) -> Option<serde_json::Value> {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(text) {
        if v.is_object() {
            return Some(v);
        }
    }

    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end <= start {
        return None;
    }

    let candidate = &text[start..=end];
    serde_json::from_str::<serde_json::Value>(candidate)
        .ok()
        .filter(|v| v.is_object())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_clean() {
        let input = r#"{"quality": "1080p", "relevance": 85}"#;
        let result = extract_json(input).unwrap();
        assert_eq!(result["relevance"], 85);
    }

    #[test]
    fn test_extract_json_with_surrounding_text() {
        let input = r#"Here is my analysis:
{"quality": "1080p", "languages": "English", "relevance": 90, "reason": "matches"}
Hope this helps!"#;
        let result = extract_json(input).unwrap();
        assert_eq!(result["relevance"], 90);
    }

    #[test]
    fn test_extract_json_no_json() {
        let input = "This response has no JSON.";
        assert!(extract_json(input).is_none());
    }
}
