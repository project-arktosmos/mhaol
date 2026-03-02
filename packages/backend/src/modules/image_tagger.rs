use super::{Module, ModuleManifest, ModuleSettingDef};
use crate::AppState;
use ndarray::{Array1, Array2, Array4, Axis};
use ort::session::Session;
use ort::value::Tensor;
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Default labels — identical to the Node.js image-tagger DEFAULT_LABELS.
// ---------------------------------------------------------------------------
const DEFAULT_LABELS: &[&str] = &[
    "a photo of a person",
    "a photo of a face",
    "a photo of a man",
    "a photo of a woman",
    "a photo of a child",
    "a photo of people",
    "a portrait photo",
    "a selfie",
    "a photo of a group of people",
    "a photo of a baby",
    "a family photo",
    "a photo of a dog",
    "a photo of a cat",
    "a photo of a bird",
    "a photo of an animal",
    "a photo of a pet",
    "a photo of a beach",
    "a photo of a mountain",
    "a photo of a city",
    "a photo of a forest",
    "a photo of a park",
    "a photo of a garden",
    "an indoor photo",
    "an outdoor photo",
    "a photo of a street",
    "a photo of a building",
    "a photo of food",
    "a photo of cooking",
    "a travel photo",
    "a photo of sports",
    "a photo of a concert",
    "a photo of a party",
    "a photo of a wedding",
    "a photo of a graduation",
    "a photo of a birthday",
    "a photo of a car",
    "a photo of a bicycle",
    "a photo of a flower",
    "a photo of a book",
    "a photo of a computer",
    "a photo of a phone",
    "a screenshot",
    "a photo of a document",
    "a photo of a receipt",
    "a photo of a whiteboard",
    "a photo of handwriting",
    "a meme",
    "a photo of a chart",
    "a photo of a map",
    "a photo of a sunset",
    "a photo of a sunrise",
    "a photo of snow",
    "a photo of rain",
    "a photo of the night sky",
    "a photo of the ocean",
    "a photo of artwork",
    "a painting",
    "a black and white photo",
    "an aerial view",
    "a macro photo",
];

const MODEL_REPO: &str = "Xenova/siglip-large-patch16-384";
const IMAGE_SIZE: u32 = 384;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TagResult {
    pub tag: String,
    pub score: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum TaggerError {
    #[error("Model download failed: {0}")]
    Download(String),
    #[error("Model initialization failed: {0}")]
    Init(String),
    #[error("Image processing failed: {0}")]
    Image(String),
    #[error("Inference failed: {0}")]
    Inference(String),
    #[error("Tagger not ready: {0}")]
    NotReady(String),
}

// ---------------------------------------------------------------------------
// Tagger state machine
// ---------------------------------------------------------------------------

enum TaggerState {
    Idle,
    Downloading { progress: u32 },
    Loading,
    Ready { pipeline: Arc<SiglipPipeline> },
    Error { message: String },
}

impl TaggerState {
    fn status_str(&self) -> &'static str {
        match self {
            TaggerState::Idle => "idle",
            TaggerState::Downloading { .. } => "downloading",
            TaggerState::Loading => "loading",
            TaggerState::Ready { .. } => "ready",
            TaggerState::Error { .. } => "error",
        }
    }
}

// ---------------------------------------------------------------------------
// SiglipPipeline — holds ONNX sessions and pre-computed embeddings
// ---------------------------------------------------------------------------

struct SiglipPipeline {
    // Session::run() requires &mut self, so we need Mutex for shared access
    vision_session: parking_lot::Mutex<Session>,
    label_embeddings: Array2<f32>,
    label_tags: Vec<String>,
    threshold: f64,
}

impl SiglipPipeline {
    fn run_inference(&self, image_path: &Path) -> Result<Vec<TagResult>, TaggerError> {
        let pixel_values = preprocess_image(image_path)?;

        let pixel_tensor = Tensor::from_array(pixel_values)
            .map_err(|e| TaggerError::Inference(format!("Failed to create pixel tensor: {e}")))?;

        // Run inference while holding the session lock, extract embedding, then release
        let image_embed_flat: Array1<f32> = {
            let mut session = self.vision_session.lock();
            let vision_outputs = session
                .run(ort::inputs!["pixel_values" => pixel_tensor])
                .map_err(|e| TaggerError::Inference(format!("Vision model failed: {e}")))?;

            let image_embeds = vision_outputs
                .get("image_embeds")
                .or_else(|| vision_outputs.get("last_hidden_state"))
                .ok_or_else(|| {
                    let names: Vec<String> = vision_outputs
                        .iter()
                        .map(|(name, _val)| name.to_string())
                        .collect();
                    TaggerError::Inference(format!(
                        "Vision model missing image_embeds output, got: {:?}",
                        names
                    ))
                })?;

            let image_embed_view = image_embeds
                .try_extract_array::<f32>()
                .map_err(|e| {
                    TaggerError::Inference(format!("Failed to extract image embeds: {e}"))
                })?;

            // Shape depends on model export: [1, embed_dim] or [1, seq_len, embed_dim]
            if image_embed_view.ndim() == 3 {
                let row = image_embed_view.index_axis(Axis(0), 0);
                row.index_axis(Axis(0), 0)
                    .to_owned()
                    .into_dimensionality()
                    .unwrap()
            } else {
                image_embed_view
                    .index_axis(Axis(0), 0)
                    .to_owned()
                    .into_dimensionality()
                    .unwrap()
            }
        };

        let image_embed = l2_normalize_1d(&image_embed_flat);

        // Compute similarity scores: label_embeddings @ image_embed
        let logits = self.label_embeddings.dot(&image_embed);
        let scores = softmax(logits.as_slice().unwrap());

        let mut results: Vec<TagResult> = scores
            .iter()
            .enumerate()
            .filter(|(_, &score)| score >= self.threshold as f32)
            .map(|(i, &score)| TagResult {
                tag: self.label_tags[i].clone(),
                score: (score as f64 * 1000.0).round() / 1000.0,
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        Ok(results)
    }
}

// ---------------------------------------------------------------------------
// ImageTaggerManager
// ---------------------------------------------------------------------------

pub struct ImageTaggerManager {
    state: RwLock<TaggerState>,
    init_lock: tokio::sync::Mutex<()>,
}

impl ImageTaggerManager {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(TaggerState::Idle),
            init_lock: tokio::sync::Mutex::new(()),
        }
    }

    pub fn get_status(&self) -> (bool, &'static str, u32, Option<String>) {
        let state = self.state.read();
        let ready = matches!(*state, TaggerState::Ready { .. });
        let status = state.status_str();
        let progress = match &*state {
            TaggerState::Ready { .. } => 100,
            TaggerState::Downloading { progress } => *progress,
            TaggerState::Loading => 100,
            _ => 0,
        };
        let error = match &*state {
            TaggerState::Error { message } => Some(message.clone()),
            _ => None,
        };
        (ready, status, progress, error)
    }

    pub async fn tag_image(
        &self,
        image_path: &str,
        settings_threshold: Option<f64>,
    ) -> Result<Vec<TagResult>, TaggerError> {
        self.ensure_ready(settings_threshold).await?;

        let pipeline = {
            let state = self.state.read();
            match &*state {
                TaggerState::Ready { pipeline } => Arc::clone(pipeline),
                _ => return Err(TaggerError::NotReady("Tagger not in ready state".into())),
            }
        };

        let path = PathBuf::from(image_path);
        tokio::task::spawn_blocking(move || pipeline.run_inference(&path))
            .await
            .map_err(|e| TaggerError::Inference(format!("Task join error: {e}")))?
    }

    async fn ensure_ready(&self, threshold: Option<f64>) -> Result<(), TaggerError> {
        // Fast path: already ready
        {
            let state = self.state.read();
            if matches!(*state, TaggerState::Ready { .. }) {
                return Ok(());
            }
        }

        // Serialize initialization attempts
        let _guard = self.init_lock.lock().await;

        // Re-check after acquiring lock
        {
            let state = self.state.read();
            if matches!(*state, TaggerState::Ready { .. }) {
                return Ok(());
            }
        }

        // Reset error state to allow retry
        {
            let mut state = self.state.write();
            if matches!(*state, TaggerState::Error { .. }) {
                *state = TaggerState::Idle;
            }
        }

        let threshold = threshold.unwrap_or(0.005);

        // Download model files
        {
            let mut state = self.state.write();
            *state = TaggerState::Downloading { progress: 0 };
        }

        tracing::info!("[image-tagger] Downloading model from {MODEL_REPO}...");

        let model_paths = download_model(MODEL_REPO, &self.state).await?;

        // Load ONNX sessions and tokenizer
        {
            let mut state = self.state.write();
            *state = TaggerState::Loading;
        }

        tracing::info!("[image-tagger] Loading ONNX sessions and tokenizer...");

        let pipeline =
            tokio::task::spawn_blocking(move || load_pipeline(&model_paths, threshold))
                .await
                .map_err(|e| TaggerError::Init(format!("Task join error: {e}")))?;

        match pipeline {
            Ok(pipeline) => {
                let mut state = self.state.write();
                *state = TaggerState::Ready {
                    pipeline: Arc::new(pipeline),
                };
                tracing::info!("[image-tagger] Pipeline ready");
                Ok(())
            }
            Err(e) => {
                let msg = e.to_string();
                let mut state = self.state.write();
                *state = TaggerState::Error {
                    message: msg.clone(),
                };
                Err(e)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Model download
// ---------------------------------------------------------------------------

struct ModelPaths {
    vision_model: PathBuf,
    text_model: PathBuf,
    tokenizer: PathBuf,
}

async fn download_model(
    repo_id: &str,
    state: &RwLock<TaggerState>,
) -> Result<ModelPaths, TaggerError> {
    let api = hf_hub::api::tokio::ApiBuilder::new()
        .build()
        .map_err(|e| TaggerError::Download(format!("Failed to create HF API: {e}")))?;

    let repo = api.model(repo_id.to_string());

    let vision_model = repo
        .get("onnx/vision_model.onnx")
        .await
        .map_err(|e| TaggerError::Download(format!("Failed to download vision_model.onnx: {e}")))?;

    {
        let mut s = state.write();
        *s = TaggerState::Downloading { progress: 33 };
    }

    let text_model = repo
        .get("onnx/text_model.onnx")
        .await
        .map_err(|e| TaggerError::Download(format!("Failed to download text_model.onnx: {e}")))?;

    {
        let mut s = state.write();
        *s = TaggerState::Downloading { progress: 66 };
    }

    // tokenizer.json is a small non-LFS file; hf-hub's range-request metadata
    // can fail for these ("content-range is missing").  Try hf-hub first, then
    // fall back to a direct reqwest download.
    let tokenizer = match repo.get("tokenizer.json").await {
        Ok(path) => path,
        Err(_) => download_tokenizer_fallback(repo_id, &vision_model).await?,
    };

    {
        let mut s = state.write();
        *s = TaggerState::Downloading { progress: 100 };
    }

    Ok(ModelPaths {
        vision_model,
        text_model,
        tokenizer,
    })
}

/// Download tokenizer.json directly via reqwest when hf-hub's range-request
/// metadata fails (common for small non-LFS files on HuggingFace).
async fn download_tokenizer_fallback(
    repo_id: &str,
    sibling_path: &Path,
) -> Result<PathBuf, TaggerError> {
    let url = format!(
        "https://huggingface.co/{}/resolve/main/tokenizer.json",
        repo_id
    );
    tracing::info!("[image-tagger] Falling back to direct download for tokenizer.json");

    let bytes = reqwest::get(&url)
        .await
        .map_err(|e| TaggerError::Download(format!("Failed to fetch tokenizer.json: {e}")))?
        .error_for_status()
        .map_err(|e| TaggerError::Download(format!("tokenizer.json HTTP error: {e}")))?
        .bytes()
        .await
        .map_err(|e| TaggerError::Download(format!("Failed to read tokenizer.json body: {e}")))?;

    // Save next to the other cached model files
    let dest = sibling_path
        .parent()
        .unwrap_or(Path::new("."))
        .join("tokenizer.json");
    tokio::fs::write(&dest, &bytes)
        .await
        .map_err(|e| TaggerError::Download(format!("Failed to write tokenizer.json: {e}")))?;
    Ok(dest)
}

// ---------------------------------------------------------------------------
// Pipeline loading (runs in blocking context)
// ---------------------------------------------------------------------------

fn load_pipeline(paths: &ModelPaths, threshold: f64) -> Result<SiglipPipeline, TaggerError> {
    let vision_session = Session::builder()
        .map_err(|e| TaggerError::Init(format!("Failed to create session builder: {e}")))?
        .commit_from_file(&paths.vision_model)
        .map_err(|e| TaggerError::Init(format!("Failed to load vision model: {e}")))?;

    let mut text_session = Session::builder()
        .map_err(|e| TaggerError::Init(format!("Failed to create session builder: {e}")))?
        .commit_from_file(&paths.text_model)
        .map_err(|e| TaggerError::Init(format!("Failed to load text model: {e}")))?;

    // Load tokenizer
    let mut tokenizer = tokenizers::Tokenizer::from_file(&paths.tokenizer)
        .map_err(|e| TaggerError::Init(format!("Failed to load tokenizer: {e}")))?;

    // Configure padding and truncation
    let vocab = tokenizer.get_vocab(true);
    let pad_id = vocab
        .get("</s>")
        .or_else(|| vocab.get("[PAD]"))
        .copied()
        .unwrap_or(1);
    drop(vocab);

    tokenizer.with_padding(Some(tokenizers::PaddingParams {
        strategy: tokenizers::PaddingStrategy::Fixed(64),
        pad_id,
        pad_token: "</s>".to_string(),
        ..Default::default()
    }));

    tokenizer
        .with_truncation(Some(tokenizers::TruncationParams {
            max_length: 64,
            ..Default::default()
        }))
        .map_err(|e| TaggerError::Init(format!("Failed to set truncation: {e}")))?;

    // Tokenize all labels
    let labels: Vec<&str> = DEFAULT_LABELS.to_vec();
    let encodings = tokenizer
        .encode_batch(labels, true)
        .map_err(|e| TaggerError::Init(format!("Failed to tokenize labels: {e}")))?;

    let num_labels = encodings.len();
    let seq_len = encodings[0].get_ids().len();

    // Build input tensors for text model (SigLIP only uses input_ids, no attention_mask)
    let mut input_ids_arr = Array2::<i64>::zeros((num_labels, seq_len));

    for (i, encoding) in encodings.iter().enumerate() {
        for (j, &id) in encoding.get_ids().iter().enumerate() {
            input_ids_arr[[i, j]] = id as i64;
        }
    }

    tracing::info!(
        "[image-tagger] Computing text embeddings for {} labels...",
        num_labels
    );

    let input_ids_tensor = Tensor::from_array(input_ids_arr)
        .map_err(|e| TaggerError::Init(format!("Failed to create input_ids tensor: {e}")))?;

    let text_outputs = text_session
        .run(ort::inputs!["input_ids" => input_ids_tensor])
        .map_err(|e| TaggerError::Init(format!("Text model failed: {e}")))?;

    // Try known output names
    let text_embeds = text_outputs
        .get("text_embeds")
        .or_else(|| text_outputs.get("last_hidden_state"))
        .ok_or_else(|| {
            let names: Vec<String> = text_outputs
                .iter()
                .map(|(name, _val)| name.to_string())
                .collect();
            TaggerError::Init(format!(
                "Text model missing text_embeds output, got: {:?}",
                names
            ))
        })?;

    let text_embed_view = text_embeds
        .try_extract_array::<f32>()
        .map_err(|e| TaggerError::Init(format!("Failed to extract text embeds: {e}")))?;

    // Handle different output shapes: [num_labels, embed_dim] or [num_labels, seq_len, embed_dim]
    let label_embeddings = if text_embed_view.ndim() == 3 {
        let embed_dim = text_embed_view.shape()[2];
        let mut embeds = Array2::<f32>::zeros((num_labels, embed_dim));
        for i in 0..num_labels {
            let label_embeds = text_embed_view.index_axis(Axis(0), i);
            let cls_embed = label_embeds.index_axis(Axis(0), 0);
            embeds.row_mut(i).assign(&cls_embed);
        }
        embeds
    } else {
        let shape = text_embed_view.shape();
        let mut embeds = Array2::<f32>::zeros((shape[0], shape[1]));
        embeds.assign(&text_embed_view.into_dimensionality::<ndarray::Ix2>().map_err(|e| {
            TaggerError::Init(format!("Unexpected text embed shape: {e}"))
        })?);
        embeds
    };

    // L2-normalize each label embedding
    let label_embeddings = l2_normalize_rows(&label_embeddings);

    // Derive display tags from labels
    let label_tags: Vec<String> = DEFAULT_LABELS.iter().map(|l| label_to_tag(l)).collect();

    tracing::info!(
        "[image-tagger] Pipeline loaded: {} labels, embed_dim={}",
        num_labels,
        label_embeddings.shape()[1]
    );

    Ok(SiglipPipeline {
        vision_session: parking_lot::Mutex::new(vision_session),
        label_embeddings,
        label_tags,
        threshold,
    })
}

// ---------------------------------------------------------------------------
// Image preprocessing
// ---------------------------------------------------------------------------

fn preprocess_image(path: &Path) -> Result<Array4<f32>, TaggerError> {
    let img = image::open(path)
        .map_err(|e| TaggerError::Image(format!("Failed to open {}: {e}", path.display())))?
        .into_rgb8();

    let resized = image::imageops::resize(
        &img,
        IMAGE_SIZE,
        IMAGE_SIZE,
        image::imageops::FilterType::Lanczos3,
    );

    let mut tensor = Array4::<f32>::zeros((1, 3, IMAGE_SIZE as usize, IMAGE_SIZE as usize));
    for y in 0..IMAGE_SIZE as usize {
        for x in 0..IMAGE_SIZE as usize {
            let pixel = resized.get_pixel(x as u32, y as u32);
            for c in 0..3 {
                let val = pixel[c] as f32 / 255.0;
                tensor[[0, c, y, x]] = (val - 0.5) / 0.5;
            }
        }
    }
    Ok(tensor)
}

// ---------------------------------------------------------------------------
// Math helpers
// ---------------------------------------------------------------------------

fn softmax(logits: &[f32]) -> Vec<f32> {
    let max = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = logits.iter().map(|&x| (x - max).exp()).collect();
    let sum: f32 = exps.iter().sum();
    exps.iter().map(|&x| x / sum).collect()
}

fn l2_normalize_1d(v: &Array1<f32>) -> Array1<f32> {
    let norm = v.dot(v).sqrt();
    if norm > 0.0 {
        v / norm
    } else {
        v.clone()
    }
}

fn l2_normalize_rows(m: &Array2<f32>) -> Array2<f32> {
    let mut result = m.clone();
    for mut row in result.rows_mut() {
        let norm: f32 = row.dot(&row).sqrt();
        if norm > 0.0 {
            row.mapv_inplace(|x| x / norm);
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Label → display tag conversion (mirrors Node.js labelToTag)
// ---------------------------------------------------------------------------

fn label_to_tag(label: &str) -> String {
    let s = label;
    let s = s
        .strip_prefix("a photo of the ")
        .or_else(|| s.strip_prefix("a photo of an "))
        .or_else(|| s.strip_prefix("a photo of a "))
        .or_else(|| s.strip_prefix("a photo of "))
        .or_else(|| s.strip_prefix("an "))
        .or_else(|| s.strip_prefix("a "))
        .unwrap_or(s);
    let s = s.strip_suffix(" photo").unwrap_or(s);
    s.to_string()
}

// ---------------------------------------------------------------------------
// Module trait implementation
// ---------------------------------------------------------------------------

pub struct ImageTaggerModule {
    pub manager: Arc<ImageTaggerManager>,
}

impl Module for ImageTaggerModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            name: "image-tagger".to_string(),
            version: "1.0.0".to_string(),
            description: "AI image tagger using SigLIP zero-shot classification".to_string(),
            source: Some("module".to_string()),
            compatibility: None,
            settings: vec![
                ModuleSettingDef {
                    key: "image-tagger.threshold".to_string(),
                    default: "0.005".to_string(),
                    env_key: Some("IMAGE_TAGGER_THRESHOLD".to_string()),
                },
                ModuleSettingDef {
                    key: "image-tagger.cacheDir".to_string(),
                    default: String::new(),
                    env_key: Some("IMAGE_TAGGER_CACHE_DIR".to_string()),
                },
            ],
            link_sources: Vec::new(),
            schema_sql: None,
        }
    }

    fn initialize(&self, _state: &AppState) -> Result<(), String> {
        tracing::info!("[image-tagger] Module registered (model loads on first use)");
        Ok(())
    }

    fn shutdown(&self) {
        tracing::info!("[image-tagger] Shutting down");
    }
}
