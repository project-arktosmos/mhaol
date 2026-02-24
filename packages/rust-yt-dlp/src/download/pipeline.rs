use anyhow::Result;
use parking_lot::Mutex;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::watch;

use crate::download::format::{select_formats, SelectedFormats};
use crate::download::http::{download_with_progress, DownloadProgressUpdate};
use crate::download::muxer::FfmpegMuxer;
use crate::error::YtDlpError;
use crate::extractor::clients::CLIENT_PRIORITY;
use crate::extractor::innertube::InnertubeApi;
use crate::extractor::player::{PlayerResponse, ResolvedFormat, StreamFormat, extract_player_js_url};
use crate::extractor::signatures::{self, SignatureResolver};
use crate::types::{AudioFormat, AudioQuality, DownloadMode, VideoFormat, VideoQuality, VideoInfo};
use crate::util::sanitize_title;

/// Configuration for a single download task.
#[derive(Debug, Clone)]
pub struct DownloadTaskConfig {
    pub video_id: String,
    pub title: String,
    pub mode: DownloadMode,
    pub audio_quality: AudioQuality,
    pub audio_format: AudioFormat,
    pub video_quality: Option<VideoQuality>,
    pub video_format: Option<VideoFormat>,
    pub output_dir: String,
}

/// Describes the progress of the pipeline stages.
#[derive(Debug, Clone)]
pub enum PipelineState {
    Fetching,
    Downloading { downloaded: u64, total: u64 },
    Muxing,
    Completed { output_path: String },
    Failed { error: String },
}

/// Orchestrates the full download pipeline.
pub struct DownloadPipeline {
    innertube: Arc<InnertubeApi>,
    sig_resolver: Arc<Mutex<SignatureResolver>>,
    muxer: Arc<FfmpegMuxer>,
    http_client: reqwest::Client,
}

impl DownloadPipeline {
    pub fn new(
        innertube: Arc<InnertubeApi>,
        sig_resolver: Arc<Mutex<SignatureResolver>>,
        muxer: Arc<FfmpegMuxer>,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .build()
            .expect("Failed to build HTTP client");
        Self {
            innertube,
            sig_resolver,
            muxer,
            http_client,
        }
    }

    /// Fetch video info without downloading.
    pub async fn fetch_video_info(
        &self,
        video_id: &str,
        po_token: Option<&str>,
    ) -> Result<VideoInfo> {
        let player_response = self.fetch_player_response(video_id, po_token).await?;

        let details = player_response
            .video_details
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No video details in player response"))?;

        let duration: f64 = details.length_seconds.parse().unwrap_or(0.0);

        Ok(VideoInfo {
            title: details.title.clone(),
            duration,
            thumbnail_url: player_response.thumbnail_url(),
            uploader: details.author.clone(),
            video_id: details.video_id.clone(),
        })
    }

    /// Execute the full download pipeline.
    pub async fn execute(
        &self,
        config: &DownloadTaskConfig,
        state_tx: watch::Sender<PipelineState>,
        cancel_rx: watch::Receiver<bool>,
    ) -> Result<String> {
        let _ = state_tx.send(PipelineState::Fetching);

        // Step 1: Get player response
        let player_response = self.fetch_player_response(&config.video_id, None).await?;

        if !player_response.is_playable() {
            let reason = player_response
                .unplayable_reason()
                .unwrap_or_else(|| "Unknown reason".to_string());
            return Err(YtDlpError::VideoUnavailable { reason }.into());
        }

        // Step 2: Resolve format URLs (handle signatures if needed)
        let resolved_formats = self
            .resolve_formats(&player_response, &config.video_id)
            .await?;

        if resolved_formats.is_empty() {
            return Err(YtDlpError::NoSuitableFormat.into());
        }

        // Step 3: Select formats
        let selected = select_formats(
            &resolved_formats,
            &config.mode,
            &config.audio_quality,
            &config.audio_format,
            config.video_quality.as_ref(),
            config.video_format.as_ref(),
        )?;

        // Step 4: Download
        let safe_title = sanitize_title(&config.title);
        let output_dir = Path::new(&config.output_dir);
        let final_output = output_dir.join(format!("{}.{}", safe_title, selected.output_extension));

        // Check cancellation
        if *cancel_rx.borrow() {
            return Err(YtDlpError::Cancelled.into());
        }

        if selected.needs_muxing {
            self.download_and_mux(config, &selected, output_dir, &final_output, &state_tx, cancel_rx)
                .await?;
        } else {
            self.download_single(&selected.audio, &final_output, config, &state_tx, cancel_rx)
                .await?;
        }

        // Step 5: Handle format conversion if needed (e.g., AAC → MP3)
        let actual_output = if config.mode == DownloadMode::Audio
            && config.audio_format == AudioFormat::Mp3
            && selected.audio.codec != "mp3"
        {
            let mp3_path = output_dir.join(format!("{}.mp3", safe_title));
            if self.muxer.is_available() {
                self.muxer
                    .convert_audio(
                        &final_output,
                        &mp3_path,
                        &AudioFormat::Mp3,
                        &config.audio_quality,
                    )
                    .await?;
                // Remove the intermediate file
                let _ = tokio::fs::remove_file(&final_output).await;
                mp3_path
            } else {
                final_output
            }
        } else {
            final_output
        };

        let output_str = actual_output.to_string_lossy().to_string();
        let _ = state_tx.send(PipelineState::Completed {
            output_path: output_str.clone(),
        });

        Ok(output_str)
    }

    async fn fetch_player_response(
        &self,
        video_id: &str,
        po_token: Option<&str>,
    ) -> Result<PlayerResponse> {
        // Try clients in priority order
        for client in CLIENT_PRIORITY {
            match self.innertube.player(video_id, client, po_token).await {
                Ok(resp) if resp.is_playable() => {
                    log::info!(
                        "Got playable response from {} client for {}",
                        client.name,
                        video_id
                    );
                    return Ok(resp);
                }
                Ok(resp) => {
                    let reason = resp.unplayable_reason().unwrap_or_default();
                    log::warn!(
                        "{} client returned unplayable for {}: {}",
                        client.name,
                        video_id,
                        reason
                    );
                }
                Err(e) => {
                    log::warn!("{} client failed for {}: {}", client.name, video_id, e);
                }
            }
        }

        Err(YtDlpError::VideoNotFound {
            video_id: video_id.to_string(),
        }
        .into())
    }

    async fn resolve_formats(
        &self,
        player_response: &PlayerResponse,
        video_id: &str,
    ) -> Result<Vec<ResolvedFormat>> {
        let raw_formats = player_response.all_formats();
        let mut resolved = Vec::new();
        let mut needs_js = false;

        // First pass: collect formats with direct URLs
        for fmt in &raw_formats {
            if let Some(url) = &fmt.url {
                resolved.push(fmt.to_resolved(url.clone()));
            } else if fmt.signature_cipher.is_some() {
                needs_js = true;
            }
        }

        // Second pass: handle signature ciphers if needed
        if needs_js {
            match self.resolve_signature_formats(&raw_formats, video_id).await {
                Ok(sig_formats) => resolved.extend(sig_formats),
                Err(e) => {
                    log::warn!("Signature resolution failed: {}", e);
                    // Continue with whatever formats we already have
                }
            }
        }

        // Apply n-parameter transformation to all resolved URLs
        let resolver = self.sig_resolver.lock();
        for fmt in &mut resolved {
            if let Ok(new_url) = signatures::apply_n_param(&fmt.url, &resolver) {
                fmt.url = new_url;
            }
        }

        Ok(resolved)
    }

    async fn resolve_signature_formats(
        &self,
        raw_formats: &[&StreamFormat],
        video_id: &str,
    ) -> Result<Vec<ResolvedFormat>> {
        // Ensure we have the player.js loaded
        self.ensure_player_js_loaded(video_id).await?;

        let resolver = self.sig_resolver.lock();
        let mut resolved = Vec::new();

        for fmt in raw_formats {
            if let Some(cipher_parts) = fmt.parse_signature_cipher() {
                match resolver.decrypt_signature(&cipher_parts.encrypted_sig) {
                    Ok(decrypted_sig) => {
                        let url = format!(
                            "{}&{}={}",
                            cipher_parts.base_url, cipher_parts.sig_param, decrypted_sig
                        );
                        resolved.push(fmt.to_resolved(url));
                    }
                    Err(e) => {
                        log::warn!("Failed to decrypt signature for itag {}: {}", fmt.itag, e);
                    }
                }
            }
        }

        Ok(resolved)
    }

    async fn ensure_player_js_loaded(&self, video_id: &str) -> Result<()> {
        // Fetch the watch page to get player.js URL
        let html = self.innertube.fetch_watch_page(video_id).await?;
        let player_js_url = extract_player_js_url(&html)?;

        {
            let resolver = self.sig_resolver.lock();
            if resolver.is_loaded_for(&player_js_url) {
                return Ok(());
            }
        }

        // Fetch and parse player.js
        let player_js_source = self.innertube.fetch_player_js(&player_js_url).await?;

        let mut resolver = self.sig_resolver.lock();
        resolver.load_player_js(&player_js_url, &player_js_source)?;

        Ok(())
    }

    async fn download_single(
        &self,
        format: &ResolvedFormat,
        output_path: &Path,
        _config: &DownloadTaskConfig,
        state_tx: &watch::Sender<PipelineState>,
        cancel_rx: watch::Receiver<bool>,
    ) -> Result<()> {
        let (progress_tx, mut progress_rx) = watch::channel(DownloadProgressUpdate {
            downloaded_bytes: 0,
            total_bytes: 0,
        });

        // Forward progress updates
        let state_tx_clone = state_tx.clone();
        let progress_forwarder = tokio::spawn(async move {
            while progress_rx.changed().await.is_ok() {
                let update = progress_rx.borrow().clone();
                let _ = state_tx_clone.send(PipelineState::Downloading {
                    downloaded: update.downloaded_bytes,
                    total: update.total_bytes,
                });
            }
        });

        download_with_progress(
            &self.http_client,
            &format.url,
            output_path,
            format.content_length,
            progress_tx,
            cancel_rx,
        )
        .await?;

        progress_forwarder.abort();
        Ok(())
    }

    async fn download_and_mux(
        &self,
        config: &DownloadTaskConfig,
        selected: &SelectedFormats,
        output_dir: &Path,
        final_output: &Path,
        state_tx: &watch::Sender<PipelineState>,
        cancel_rx: watch::Receiver<bool>,
    ) -> Result<()> {
        let safe_title = sanitize_title(&config.title);
        let video_format = selected.video.as_ref().unwrap();
        let audio_format = &selected.audio;

        let video_tmp = output_dir.join(format!("{}.video.tmp.{}", safe_title, video_format.container));
        let audio_tmp = output_dir.join(format!("{}.audio.tmp.{}", safe_title, audio_format.container));

        // Download video
        let (progress_tx, mut progress_rx) = watch::channel(DownloadProgressUpdate {
            downloaded_bytes: 0,
            total_bytes: 0,
        });

        let state_tx_clone = state_tx.clone();
        let progress_forwarder = tokio::spawn(async move {
            while progress_rx.changed().await.is_ok() {
                let update = progress_rx.borrow().clone();
                let _ = state_tx_clone.send(PipelineState::Downloading {
                    downloaded: update.downloaded_bytes,
                    total: update.total_bytes,
                });
            }
        });

        download_with_progress(
            &self.http_client,
            &video_format.url,
            &video_tmp,
            video_format.content_length,
            progress_tx,
            cancel_rx.clone(),
        )
        .await?;

        progress_forwarder.abort();

        // Check cancellation before audio download
        if *cancel_rx.borrow() {
            let _ = tokio::fs::remove_file(&video_tmp).await;
            return Err(YtDlpError::Cancelled.into());
        }

        // Download audio
        let (progress_tx2, _progress_rx2) = watch::channel(DownloadProgressUpdate {
            downloaded_bytes: 0,
            total_bytes: 0,
        });

        download_with_progress(
            &self.http_client,
            &audio_format.url,
            &audio_tmp,
            audio_format.content_length,
            progress_tx2,
            cancel_rx,
        )
        .await?;

        // Mux
        let _ = state_tx.send(PipelineState::Muxing);

        let video_fmt = config
            .video_format
            .clone()
            .unwrap_or(crate::types::VideoFormat::Mp4);

        if self.muxer.is_available() {
            self.muxer
                .mux(&video_tmp, &audio_tmp, final_output, &video_fmt)
                .await?;
        } else {
            // If ffmpeg isn't available, just use the video file as output
            log::warn!("FFmpeg not available, output will be video-only");
            tokio::fs::rename(&video_tmp, final_output).await?;
        }

        // Clean up temp files
        let _ = tokio::fs::remove_file(&video_tmp).await;
        let _ = tokio::fs::remove_file(&audio_tmp).await;

        Ok(())
    }
}
