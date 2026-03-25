use crate::AppState;
use futures_util::StreamExt;
use mhaol_llm::{build_chat_prompt, list_models, load_model_blocking, run_inference_blocking, ChatMessage, LlmTokenEvent};
use tokio::io::AsyncWriteExt;
use tracing::{info, warn, error};

const DEFAULT_MODEL_REPO: &str = "Qwen/Qwen2.5-1.5B-Instruct-GGUF";
const DEFAULT_MODEL_FILE: &str = "qwen2.5-1.5b-instruct-q4_k_m.gguf";

/// Download the default GGUF model from HuggingFace if no models are available.
async fn download_default_model(state: &AppState) -> Result<String, String> {
    let models_dir = &state.llm_engine.models_dir;
    let dest_path = models_dir.join(DEFAULT_MODEL_FILE);

    if dest_path.exists() {
        return Ok(DEFAULT_MODEL_FILE.to_string());
    }

    let url = format!(
        "https://huggingface.co/{}/resolve/main/{}",
        DEFAULT_MODEL_REPO, DEFAULT_MODEL_FILE
    );

    info!("[llm-worker] Downloading default model from {}", url);

    let downloads_dir = models_dir.parent().unwrap_or(models_dir).join("downloads");
    std::fs::create_dir_all(&downloads_dir)
        .map_err(|e| format!("Failed to create downloads dir: {}", e))?;

    let temp_path = downloads_dir.join(DEFAULT_MODEL_FILE);

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await
        .map_err(|e| format!("Download request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed: HTTP {}", response.status()));
    }

    let total_bytes = response.content_length().unwrap_or(0);
    info!(
        "[llm-worker] Downloading {} ({:.1} MB)",
        DEFAULT_MODEL_FILE,
        total_bytes as f64 / 1_048_576.0
    );

    let mut file = tokio::fs::File::create(&temp_path).await
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    let mut byte_stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut last_log = std::time::Instant::now();

    while let Some(chunk_result) = byte_stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Download stream error: {}", e))?;
        file.write_all(&chunk).await
            .map_err(|e| format!("Failed to write chunk: {}", e))?;
        downloaded += chunk.len() as u64;

        if last_log.elapsed() > std::time::Duration::from_secs(10) {
            let pct = if total_bytes > 0 {
                (downloaded as f64 / total_bytes as f64) * 100.0
            } else {
                0.0
            };
            info!("[llm-worker] Download progress: {:.1}% ({:.1} MB)", pct, downloaded as f64 / 1_048_576.0);
            last_log = std::time::Instant::now();
        }
    }

    file.flush().await.map_err(|e| format!("Flush failed: {}", e))?;

    std::fs::rename(&temp_path, &dest_path)
        .map_err(|e| format!("Failed to move downloaded model: {}", e))?;

    info!("[llm-worker] Download complete: {}", DEFAULT_MODEL_FILE);
    Ok(DEFAULT_MODEL_FILE.to_string())
}

/// Ensure a model is loaded. Downloads default model if none available, then loads it.
async fn ensure_model_loaded(state: &AppState) {
    if state.llm_engine.is_model_loaded() {
        return;
    }

    let mut models = list_models(&state.llm_engine);

    // No models on disk — download the default one
    if models.is_empty() {
        match download_default_model(state).await {
            Ok(_) => {
                models = list_models(&state.llm_engine);
            }
            Err(e) => {
                error!("[llm-worker] Failed to download default model: {}", e);
                return;
            }
        }
    }

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
                    "llm:extract-show-info" => process_extract_show_info(&state, &task).await,
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

async fn process_extract_show_info(state: &AppState, task: &mhaol_queue::QueueTask) {
    let payload = &task.payload;
    let folder_name = payload["folderName"].as_str().unwrap_or("");

    if folder_name.is_empty() {
        state.queue.fail(&task.id, "Missing folderName in payload");
        return;
    }

    if !state.llm_engine.is_model_loaded() {
        state.queue.fail(&task.id, "No LLM model available");
        return;
    }

    let prompt_text = format!(
        "Extract the TV show name, release year, and season number from this folder/torrent name.\n\
         Respond with JSON only, no other text.\n\n\
         Folder: \"{}\"\n\n\
         {{\"showName\": \"<name>\", \"year\": <year or null>, \"season\": <number or null>}}",
        folder_name
    );

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: prompt_text,
    }];

    let system_prompt = "You extract structured metadata from media folder names. Respond in JSON only.";
    let prompt = build_chat_prompt(&messages, system_prompt);

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
        run_inference_blocking(model, prompt, 128, cancel_flag, tx);
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
        state.queue.complete(
            &task.id,
            serde_json::json!({ "showName": folder_name, "year": null, "season": null }),
        );
        return;
    }

    let json_result = extract_json(&full_response);

    match json_result {
        Some(parsed) if parsed.get("showName").and_then(|v| v.as_str()).is_some() => {
            info!("[llm-worker] Task {} completed: extracted {:?}", task.id, parsed);
            state.queue.complete(&task.id, parsed);
        }
        _ => {
            warn!("[llm-worker] Task {} — could not parse show info, using raw name", task.id);
            state.queue.complete(
                &task.id,
                serde_json::json!({ "showName": folder_name, "year": null, "season": null }),
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
