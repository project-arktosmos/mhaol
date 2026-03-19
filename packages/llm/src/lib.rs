#[cfg(not(target_os = "android"))]
use llama_cpp::standard_sampler::StandardSampler;
#[cfg(not(target_os = "android"))]
use llama_cpp::{LlamaModel, LlamaParams, SessionParams};
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{error, info};

/// LLM configuration settings.
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: u32,
    pub repeat_penalty: f32,
    pub max_tokens: usize,
    pub system_prompt: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        LlmConfig {
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            repeat_penalty: 1.1,
            max_tokens: 2048,
            system_prompt: "You are a helpful assistant.".to_string(),
        }
    }
}

/// LLM engine state — manages model and configuration.
pub struct LlmEngine {
    #[cfg(not(target_os = "android"))]
    model: RwLock<Option<Arc<LlamaModel>>>,
    current_model_name: Mutex<Option<String>>,
    cancel_flag: Arc<AtomicBool>,
    config: RwLock<LlmConfig>,
    pub models_dir: PathBuf,
}

impl LlmEngine {
    pub fn new(models_dir: PathBuf) -> Self {
        if !models_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&models_dir) {
                error!("[LLM] Failed to create models directory: {}", e);
            }
        }

        info!(
            "[LLM] Initializing LlmEngine with models dir: {:?}",
            models_dir
        );

        LlmEngine {
            #[cfg(not(target_os = "android"))]
            model: RwLock::new(None),
            current_model_name: Mutex::new(None),
            cancel_flag: Arc::new(AtomicBool::new(false)),
            config: RwLock::new(LlmConfig::default()),
            models_dir,
        }
    }

    pub fn is_model_loaded(&self) -> bool {
        #[cfg(not(target_os = "android"))]
        {
            self.model.read().is_some()
        }
        #[cfg(target_os = "android")]
        {
            false
        }
    }

    #[cfg(not(target_os = "android"))]
    pub fn get_model(&self) -> Result<Arc<LlamaModel>, String> {
        self.model
            .read()
            .clone()
            .ok_or_else(|| "No model loaded".to_string())
    }

    pub fn get_config(&self) -> LlmConfig {
        self.config.read().clone()
    }

    pub fn get_current_model_name(&self) -> Option<String> {
        self.current_model_name.lock().clone()
    }

    #[cfg(not(target_os = "android"))]
    pub fn set_model(&self, model: Arc<LlamaModel>, name: String) {
        *self.model.write() = Some(model);
        *self.current_model_name.lock() = Some(name);
    }

    pub fn unload_model(&self) {
        #[cfg(not(target_os = "android"))]
        {
            *self.model.write() = None;
        }
        *self.current_model_name.lock() = None;
    }

    pub fn cancel_generation(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
    }

    pub fn reset_cancel_flag(&self) {
        self.cancel_flag.store(false, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(Ordering::SeqCst)
    }

    pub fn cancel_flag(&self) -> Arc<AtomicBool> {
        self.cancel_flag.clone()
    }

    pub fn update_config(&self, update: LlmConfigUpdate) {
        let mut config = self.config.write();
        if let Some(v) = update.temperature {
            config.temperature = v.clamp(0.0, 2.0);
        }
        if let Some(v) = update.top_p {
            config.top_p = v.clamp(0.0, 1.0);
        }
        if let Some(v) = update.top_k {
            config.top_k = v;
        }
        if let Some(v) = update.repeat_penalty {
            config.repeat_penalty = v;
        }
        if let Some(v) = update.max_tokens {
            config.max_tokens = v;
        }
        if let Some(v) = update.system_prompt {
            config.system_prompt = v;
        }
    }
}

// -- API types --

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmStatus {
    pub available: bool,
    pub model_loaded: bool,
    pub current_model: Option<String>,
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: u32,
    pub repeat_penalty: f32,
    pub max_tokens: usize,
    pub system_prompt: String,
    pub models_dir: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub name: String,
    pub file_name: String,
    pub size_bytes: u64,
    pub path: String,
    pub is_loaded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmTokenEvent {
    pub content: String,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmDownloadProgress {
    pub model_name: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percent: f32,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmConfigUpdate {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    pub repeat_penalty: Option<f32>,
    pub max_tokens: Option<usize>,
    pub system_prompt: Option<String>,
}

/// Build a ChatML-format prompt from messages.
pub fn build_chat_prompt(messages: &[ChatMessage], default_system: &str) -> String {
    let mut prompt = String::new();

    let has_system = messages.iter().any(|m| m.role == "system");
    if !has_system {
        prompt.push_str(&format!(
            "<|im_start|>system\n{}<|im_end|>\n",
            default_system
        ));
    }

    for message in messages {
        prompt.push_str(&format!(
            "<|im_start|>{}\n{}<|im_end|>\n",
            message.role, message.content
        ));
    }

    prompt.push_str("<|im_start|>assistant\n");
    prompt
}

/// List available .gguf models in the models directory.
pub fn list_models(engine: &LlmEngine) -> Vec<ModelInfo> {
    let models_dir = &engine.models_dir;
    if !models_dir.exists() {
        return Vec::new();
    }

    let current_model = engine.get_current_model_name();

    let mut models = Vec::new();
    let entries = match std::fs::read_dir(models_dir) {
        Ok(e) => e,
        Err(e) => {
            error!("[LLM] Failed to read models directory: {}", e);
            return Vec::new();
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        if !file_name.ends_with(".gguf") {
            continue;
        }

        let size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        let name = file_name
            .trim_end_matches(".gguf")
            .replace('-', " ")
            .replace('_', " ");

        let is_loaded = current_model.as_ref().map(|m| m == &file_name).unwrap_or(false);

        models.push(ModelInfo {
            name,
            file_name,
            size_bytes,
            path: path.display().to_string(),
            is_loaded,
        });
    }

    models.sort_by(|a, b| a.name.cmp(&b.name));
    models
}

/// Load a model from file (blocking, call from spawn_blocking).
#[cfg(not(target_os = "android"))]
pub fn load_model_blocking(model_path: PathBuf) -> Result<LlamaModel, String> {
    let params = LlamaParams::default();
    LlamaModel::load_from_file(model_path, params)
        .map_err(|e| format!("Failed to load model: {:?}", e))
}

/// Run streaming inference (blocking, sends tokens through channel).
#[cfg(not(target_os = "android"))]
pub fn run_inference_blocking(
    model: Arc<LlamaModel>,
    prompt: String,
    max_tokens: usize,
    cancel_flag: Arc<AtomicBool>,
    token_tx: tokio::sync::mpsc::Sender<LlmTokenEvent>,
) {
    let session_params = SessionParams::default();
    let mut session = match model.create_session(session_params) {
        Ok(s) => s,
        Err(e) => {
            error!("[LLM] Failed to create session: {:?}", e);
            let _ = token_tx.blocking_send(LlmTokenEvent {
                content: String::new(),
                done: true,
            });
            return;
        }
    };

    if let Err(e) = session.advance_context(prompt) {
        error!("[LLM] Failed to advance context: {:?}", e);
        let _ = token_tx.blocking_send(LlmTokenEvent {
            content: String::new(),
            done: true,
        });
        return;
    }

    let sampler = StandardSampler::default();
    let completions = match session.start_completing_with(sampler, max_tokens) {
        Ok(c) => c,
        Err(e) => {
            error!("[LLM] Failed to start completion: {:?}", e);
            let _ = token_tx.blocking_send(LlmTokenEvent {
                content: String::new(),
                done: true,
            });
            return;
        }
    };

    for token in completions.into_strings() {
        if cancel_flag.load(Ordering::SeqCst) {
            info!("[LLM] Generation cancelled");
            break;
        }

        if token_tx
            .blocking_send(LlmTokenEvent {
                content: token,
                done: false,
            })
            .is_err()
        {
            break;
        }
    }

    let _ = token_tx.blocking_send(LlmTokenEvent {
        content: String::new(),
        done: true,
    });
}
