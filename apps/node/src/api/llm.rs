use crate::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Sse,
    },
    routing::{get, post, put},
    Json, Router,
};
use futures_util::StreamExt;
use mhaol_llm::{
    list_models, load_model_blocking, LlmConfigUpdate, LlmDownloadProgress, LlmStatus,
};
use serde::Deserialize;
use std::convert::Infallible;
use tokio::io::AsyncWriteExt;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(get_status))
        .route("/models", get(get_models))
        .route("/models/load", post(load_model))
        .route("/models/unload", post(unload_model))
        .route("/models/download", post(download_model))
        .route("/config", put(update_config))
}

async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    let engine = &state.llm_engine;
    let config = engine.get_config();
    Json(LlmStatus {
        available: true,
        model_loaded: engine.is_model_loaded(),
        current_model: engine.get_current_model_name(),
        temperature: config.temperature,
        top_p: config.top_p,
        top_k: config.top_k,
        repeat_penalty: config.repeat_penalty,
        max_tokens: config.max_tokens,
        system_prompt: config.system_prompt,
        models_dir: engine.models_dir.display().to_string(),
        error: None,
    })
}

async fn get_models(State(state): State<AppState>) -> impl IntoResponse {
    Json(list_models(&state.llm_engine))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoadModelRequest {
    file_name: String,
}

async fn load_model(
    State(state): State<AppState>,
    Json(body): Json<LoadModelRequest>,
) -> impl IntoResponse {
    let model_path = state.llm_engine.models_dir.join(&body.file_name);
    if !model_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("Model not found: {}", body.file_name) })),
        )
            .into_response();
    }

    let path_clone = model_path.clone();
    let result = tokio::task::spawn_blocking(move || load_model_blocking(path_clone)).await;

    match result {
        Ok(Ok(model)) => {
            state
                .llm_engine
                .set_model(std::sync::Arc::new(model), body.file_name);
            Json(serde_json::json!({ "ok": true })).into_response()
        }
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Task error: {:?}", e) })),
        )
            .into_response(),
    }
}

async fn unload_model(State(state): State<AppState>) -> impl IntoResponse {
    state.llm_engine.unload_model();
    Json(serde_json::json!({ "ok": true }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DownloadModelRequest {
    repo_id: String,
    file_name: String,
}

async fn download_model(
    State(state): State<AppState>,
    Json(body): Json<DownloadModelRequest>,
) -> Result<
    Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>,
    (StatusCode, Json<serde_json::Value>),
> {
    let url = format!(
        "https://huggingface.co/{}/resolve/main/{}",
        body.repo_id, body.file_name
    );
    let dest_path = state.llm_engine.models_dir.join(&body.file_name);

    let downloads_dir = state
        .llm_engine
        .models_dir
        .parent()
        .unwrap()
        .join("downloads");
    if !downloads_dir.exists() {
        std::fs::create_dir_all(&downloads_dir).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("Failed to create downloads dir: {}", e) })),
            )
        })?;
    }

    let temp_path = downloads_dir.join(&body.file_name);
    let file_name = body.file_name.clone();

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": format!("Download failed: {}", e) })),
        )
    })?;

    if !response.status().is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({ "error": format!("HTTP {}", response.status()) })),
        ));
    }

    let total_bytes = response.content_length().unwrap_or(0);

    let stream = async_stream::stream! {
        let mut downloaded_bytes: u64 = 0;
        let mut file = match tokio::fs::File::create(&temp_path).await {
            Ok(f) => f,
            Err(e) => {
                let progress = LlmDownloadProgress {
                    model_name: file_name.clone(),
                    downloaded_bytes: 0,
                    total_bytes,
                    percent: 0.0,
                    status: format!("error: {}", e),
                };
                if let Ok(json) = serde_json::to_string(&progress) {
                    yield Ok(Event::default().data(json));
                }
                return;
            }
        };

        let mut byte_stream = response.bytes_stream();
        let mut last_update = std::time::Instant::now();

        while let Some(chunk_result) = byte_stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    if let Err(e) = file.write_all(&chunk).await {
                        let progress = LlmDownloadProgress {
                            model_name: file_name.clone(),
                            downloaded_bytes,
                            total_bytes,
                            percent: 0.0,
                            status: format!("error: {}", e),
                        };
                        if let Ok(json) = serde_json::to_string(&progress) {
                            yield Ok(Event::default().data(json));
                        }
                        return;
                    }
                    downloaded_bytes += chunk.len() as u64;

                    if last_update.elapsed() > std::time::Duration::from_millis(200) {
                        let percent = if total_bytes > 0 {
                            (downloaded_bytes as f32 / total_bytes as f32) * 100.0
                        } else {
                            0.0
                        };
                        let progress = LlmDownloadProgress {
                            model_name: file_name.clone(),
                            downloaded_bytes,
                            total_bytes,
                            percent,
                            status: "downloading".to_string(),
                        };
                        if let Ok(json) = serde_json::to_string(&progress) {
                            yield Ok(Event::default().data(json));
                        }
                        last_update = std::time::Instant::now();
                    }
                }
                Err(e) => {
                    let progress = LlmDownloadProgress {
                        model_name: file_name.clone(),
                        downloaded_bytes,
                        total_bytes,
                        percent: 0.0,
                        status: format!("error: {}", e),
                    };
                    if let Ok(json) = serde_json::to_string(&progress) {
                        yield Ok(Event::default().data(json));
                    }
                    return;
                }
            }
        }

        let _ = file.flush().await;

        // Move from temp to final location
        if let Err(e) = std::fs::rename(&temp_path, &dest_path) {
            let progress = LlmDownloadProgress {
                model_name: file_name.clone(),
                downloaded_bytes,
                total_bytes,
                percent: 0.0,
                status: format!("error: {}", e),
            };
            if let Ok(json) = serde_json::to_string(&progress) {
                yield Ok(Event::default().data(json));
            }
            return;
        }

        let progress = LlmDownloadProgress {
            model_name: file_name.clone(),
            downloaded_bytes: total_bytes,
            total_bytes,
            percent: 100.0,
            status: "complete".to_string(),
        };
        if let Ok(json) = serde_json::to_string(&progress) {
            yield Ok(Event::default().data(json));
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

async fn update_config(
    State(state): State<AppState>,
    Json(body): Json<LlmConfigUpdate>,
) -> impl IntoResponse {
    state.llm_engine.update_config(body);
    Json(serde_json::json!({ "ok": true }))
}
