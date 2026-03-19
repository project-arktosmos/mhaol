use anyhow::Result;
use parking_lot::Mutex;
use reqwest::header::{HeaderMap, HeaderValue, ORIGIN, REFERER, USER_AGENT};
use serde::Serialize;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::watch;

use crate::download::format::{select_formats, SelectedFormats};
use crate::download::http::{download_with_progress, DownloadProgressUpdate};
use crate::download::muxer::FfmpegMuxer;
use crate::error::YtDlpError;
use crate::extractor::clients::{InnertubeClient, CLIENT_PRIORITY, WEB};
use crate::extractor::innertube::InnertubeApi;
use crate::extractor::player::{PlayerResponse, ResolvedFormat, StreamFormat, extract_player_js_url};
use crate::extractor::signatures::{self, SignatureResolver};
use crate::types::{AudioFormat, AudioQuality, DownloadMode, VideoFormat, VideoQuality, VideoInfo};

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
    /// When set, video files are saved here instead of output_dir (used in Both mode).
    pub video_output_dir: Option<String>,
    /// When set, audio files are saved here instead of output_dir (used in Both mode).
    pub audio_output_dir: Option<String>,
    pub po_token: Option<String>,
    pub visitor_data: Option<String>,
}

/// Result of a completed download pipeline.
#[derive(Debug, Clone)]
pub struct PipelineOutput {
    /// Primary output path (for Audio or Video mode).
    pub output_path: String,
    /// Path to the downloaded video file (set in Both/Video mode).
    pub video_output_path: Option<String>,
    /// Path to the downloaded audio file (set in Both/Audio mode).
    pub audio_output_path: Option<String>,
}

/// Describes the progress of the pipeline stages.
#[derive(Debug, Clone)]
pub enum PipelineState {
    Fetching,
    Downloading { downloaded: u64, total: u64, video_path: Option<String> },
    Muxing,
    Completed { output: PipelineOutput },
    Failed { error: String },
}

/// Result of stream URL extraction (without downloading).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamUrlResult {
    pub formats: Vec<ResolvedFormat>,
    pub expires_at: u64,
}

/// Orchestrates the full download pipeline.
pub struct DownloadPipeline {
    innertube: Arc<InnertubeApi>,
    sig_resolver: Arc<Mutex<SignatureResolver>>,
    muxer: Arc<FfmpegMuxer>,
}

impl DownloadPipeline {
    pub fn new(
        innertube: Arc<InnertubeApi>,
        sig_resolver: Arc<Mutex<SignatureResolver>>,
        muxer: Arc<FfmpegMuxer>,
    ) -> Self {
        Self {
            innertube,
            sig_resolver,
            muxer,
        }
    }

    /// Fetch video info without downloading.
    pub async fn fetch_video_info(
        &self,
        video_id: &str,
        po_token: Option<&str>,
        visitor_data: Option<&str>,
    ) -> Result<VideoInfo> {
        let (player_response, _client, _token_used) = self.fetch_player_response(video_id, po_token, visitor_data).await?;

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

    /// Extract resolved stream URLs without downloading.
    pub async fn extract_stream_urls(
        &self,
        video_id: &str,
        po_token: Option<&str>,
        visitor_data: Option<&str>,
    ) -> Result<StreamUrlResult> {
        let (player_response, _client, po_token_was_used) =
            self.fetch_player_response(video_id, po_token, visitor_data).await?;

        if !player_response.is_playable() {
            let reason = player_response
                .unplayable_reason()
                .unwrap_or_else(|| "Unknown reason".to_string());
            return Err(YtDlpError::VideoUnavailable { reason }.into());
        }

        let mut resolved_formats = self.resolve_formats(&player_response, video_id).await?;

        if resolved_formats.is_empty() {
            return Err(YtDlpError::NoSuitableFormat.into());
        }

        // Append pot=TOKEN when applicable
        if po_token_was_used {
            if let Some(token) = po_token {
                for fmt in &mut resolved_formats {
                    if fmt.url.contains('?') {
                        fmt.url = format!("{}&pot={}", fmt.url, token);
                    } else {
                        fmt.url = format!("{}?pot={}", fmt.url, token);
                    }
                }
            }
        }

        let expires_in = player_response
            .streaming_data
            .as_ref()
            .and_then(|sd| sd.expires_in_seconds.as_ref())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(21600); // default 6 hours

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(StreamUrlResult {
            formats: resolved_formats,
            expires_at: now + expires_in,
        })
    }

    /// Execute the full download pipeline.
    pub async fn execute(
        &self,
        config: &DownloadTaskConfig,
        state_tx: watch::Sender<PipelineState>,
        cancel_rx: watch::Receiver<bool>,
    ) -> Result<PipelineOutput> {
        let _ = state_tx.send(PipelineState::Fetching);

        // Step 1: Get player response
        let (player_response, client, po_token_was_used) = self.fetch_player_response(
            &config.video_id,
            config.po_token.as_deref(),
            config.visitor_data.as_deref(),
        ).await?;

        if !player_response.is_playable() {
            let reason = player_response
                .unplayable_reason()
                .unwrap_or_else(|| "Unknown reason".to_string());
            return Err(YtDlpError::VideoUnavailable { reason }.into());
        }

        // Step 2: Resolve format URLs
        let mut resolved_formats = self
            .resolve_formats(&player_response, &config.video_id)
            .await?;

        if resolved_formats.is_empty() {
            return Err(YtDlpError::NoSuitableFormat.into());
        }

        // Append pot=TOKEN when applicable
        if po_token_was_used {
            if let Some(ref token) = config.po_token {
                for fmt in &mut resolved_formats {
                    if fmt.url.contains('?') {
                        fmt.url = format!("{}&pot={}", fmt.url, token);
                    } else {
                        fmt.url = format!("{}?pot={}", fmt.url, token);
                    }
                }
                log::info!("Appended pot= parameter to {} format URLs", resolved_formats.len());
            }
        } else if config.po_token.is_some() {
            log::info!(
                "Skipping pot= parameter — PO token was not used with {} client",
                client.name,
            );
        }

        // Step 3: Select formats
        let has_po_token = po_token_was_used;

        // For Both mode, treat video format selection as Video mode
        let effective_mode = match config.mode {
            DownloadMode::Both => &DownloadMode::Video,
            ref m => m,
        };
        let selected = select_formats(
            &resolved_formats,
            effective_mode,
            &config.audio_quality,
            &config.audio_format,
            config.video_quality.as_ref(),
            config.video_format.as_ref(),
            has_po_token,
        )?;

        // For Both mode, also select audio-only format for direct download (no ffmpeg needed).
        // This tries audio-only adaptive streams first; falls back to None if only muxed
        // fallback is available (so ffmpeg extraction from video is used instead).
        let audio_selected = if config.mode == DownloadMode::Both {
            select_formats(
                &resolved_formats,
                &DownloadMode::Audio,
                &config.audio_quality,
                &config.audio_format,
                None,
                None,
                has_po_token,
            )
            .ok()
            .filter(|s| !s.needs_audio_extraction)
        } else {
            None
        };

        // Step 4: Build download headers
        let download_headers = Self::build_download_headers(client);
        let http_client = self.innertube.http_client();

        let file_stem = &config.video_id;

        // Determine output directories
        let video_dir = config.video_output_dir.as_deref().unwrap_or(&config.output_dir);
        let audio_dir = config.audio_output_dir.as_deref().unwrap_or(&config.output_dir);
        let output_dir = Path::new(&config.output_dir);
        let video_output_dir = Path::new(video_dir);
        let audio_output_dir = Path::new(audio_dir);

        // Check cancellation
        if *cancel_rx.borrow() {
            return Err(YtDlpError::Cancelled.into());
        }

        match config.mode {
            DownloadMode::Both => {
                self.execute_both(
                    config, &selected, audio_selected.as_ref(),
                    output_dir, video_output_dir, audio_output_dir,
                    file_stem, &state_tx, cancel_rx, http_client, &download_headers,
                ).await
            }
            DownloadMode::Video => {
                let final_output = video_output_dir.join(format!("{}.{}", file_stem, selected.output_extension));
                self.execute_single_mode(
                    config, &selected, output_dir, &final_output, None,
                    file_stem, &state_tx, cancel_rx, http_client, &download_headers,
                ).await
            }
            DownloadMode::Audio => {
                let final_output = audio_output_dir.join(format!("{}.{}", file_stem, selected.output_extension));
                if selected.needs_audio_extraction {
                    // Muxed stream path — no change needed
                    self.execute_single_mode(
                        config, &selected, output_dir, &final_output, None,
                        file_stem, &state_tx, cancel_rx, http_client, &download_headers,
                    ).await
                } else {
                    // Audio-only adaptive stream — with WEB client fallback on CDN 403
                    let download_result = self.download_audio_with_client_fallback(
                        &selected.audio, &final_output, config, &state_tx,
                        cancel_rx.clone(), http_client, &download_headers,
                    ).await;

                    let actual_output = match download_result {
                        Ok(()) => final_output.clone(),
                        Err(e) => {
                            // Audio-only streams failed even with WEB client — fall back to
                            // muxed stream download + ffmpeg audio extraction
                            log::warn!(
                                "Audio-only stream failed ({}), falling back to muxed stream + ffmpeg extraction",
                                e
                            );
                            let muxed_formats: Vec<_> = resolved_formats
                                .iter()
                                .filter(|f| !f.is_audio_only && !f.is_video_only)
                                .cloned()
                                .collect();
                            if muxed_formats.is_empty() {
                                return Err(e);
                            }
                            let muxed = select_formats(
                                &muxed_formats,
                                &DownloadMode::Audio,
                                &config.audio_quality,
                                &config.audio_format,
                                None,
                                None,
                                false,
                            )?;
                            let muxed_tmp = audio_output_dir
                                .join(format!("{}.muxed.tmp.{}", file_stem, muxed.output_extension));
                            self.download_single_with_retry(
                                &muxed.audio, &muxed_tmp, false, config, &state_tx,
                                cancel_rx.clone(), http_client, &download_headers,
                            ).await?;
                            if self.muxer.is_available() {
                                let audio_ext = match config.audio_format {
                                    AudioFormat::Aac => "m4a",
                                    AudioFormat::Mp3 => "mp3",
                                    AudioFormat::Opus => "opus",
                                };
                                let audio_path = audio_output_dir
                                    .join(format!("{}.{}", file_stem, audio_ext));
                                let _ = state_tx.send(PipelineState::Muxing);
                                self.muxer
                                    .convert_audio(
                                        &muxed_tmp,
                                        &audio_path,
                                        &config.audio_format,
                                        &config.audio_quality,
                                    )
                                    .await?;
                                let _ = tokio::fs::remove_file(&muxed_tmp).await;
                                audio_path
                            } else {
                                log::warn!("FFmpeg not available; returning muxed file as audio fallback");
                                muxed_tmp
                            }
                        }
                    };

                    // Convert to mp3 if the stream codec doesn't match
                    let final_path = if config.audio_format == AudioFormat::Mp3
                        && selected.audio.codec != "mp3"
                        && actual_output == final_output
                    {
                        let mp3_path = audio_output_dir.join(format!("{}.mp3", file_stem));
                        if self.muxer.is_available() {
                            self.muxer
                                .convert_audio(
                                    &actual_output,
                                    &mp3_path,
                                    &AudioFormat::Mp3,
                                    &config.audio_quality,
                                )
                                .await?;
                            let _ = tokio::fs::remove_file(&actual_output).await;
                            mp3_path
                        } else {
                            actual_output
                        }
                    } else {
                        actual_output
                    };

                    let output_str = final_path.to_string_lossy().to_string();
                    let output = PipelineOutput {
                        output_path: output_str.clone(),
                        video_output_path: None,
                        audio_output_path: Some(output_str),
                    };
                    let _ = state_tx.send(PipelineState::Completed { output: output.clone() });
                    Ok(output)
                }
            }
        }
    }

    /// Execute download in Both mode: produce both a video and audio file.
    ///
    /// Audio strategy (in priority order):
    ///   1. Direct download of an audio-only adaptive stream (`audio_selected` is Some) — no ffmpeg needed.
    ///   2. Extract audio from the downloaded video file via ffmpeg.
    ///   3. Skip audio (ffmpeg unavailable and no audio-only stream).
    async fn execute_both(
        &self,
        config: &DownloadTaskConfig,
        selected: &SelectedFormats,
        audio_selected: Option<&SelectedFormats>,
        output_dir: &Path,
        video_output_dir: &Path,
        audio_output_dir: &Path,
        file_stem: &str,
        state_tx: &watch::Sender<PipelineState>,
        cancel_rx: watch::Receiver<bool>,
        http_client: &reqwest::Client,
        download_headers: &HeaderMap,
    ) -> Result<PipelineOutput> {
        let video_final = video_output_dir.join(format!("{}.{}", file_stem, selected.output_extension));

        // Download and produce the video file
        if selected.needs_muxing {
            self.download_and_mux(config, selected, output_dir, &video_final, state_tx, cancel_rx.clone(), http_client, download_headers)
                .await?;
        } else {
            // Muxed stream — download it as the video file (expose path for streaming)
            self.download_single_with_retry(&selected.audio, &video_final, true, config, state_tx, cancel_rx.clone(), http_client, download_headers)
                .await?;
        }

        let video_path_str = video_final.to_string_lossy().to_string();

        // Produce the audio file
        let audio_path_str = if let Some(audio) = audio_selected {
            // Strategy 1: direct audio-only stream download (preferred, no ffmpeg needed).
            // On CDN 403 (e.g. ANDROID throttles adaptive), retry with WEB client before
            // falling through to strategy 2.
            let audio_final = audio_output_dir.join(format!("{}.{}", file_stem, audio.output_extension));
            log::info!(
                "Both mode: downloading audio-only stream ({} container) to {:?}",
                audio.output_extension,
                audio_final
            );
            match self.download_audio_with_client_fallback(
                &audio.audio, &audio_final, config, state_tx, cancel_rx.clone(), http_client, download_headers,
            ).await {
                Ok(()) => audio_final.to_string_lossy().to_string(),
                Err(e) => {
                    log::warn!("Both mode: audio-only stream failed ({}), falling through to ffmpeg extraction", e);
                    // Fall through to strategy 2 below by treating this as if audio_selected was None
                    if self.muxer.is_available() {
                        let audio_ext = match config.audio_format {
                            AudioFormat::Aac => "m4a",
                            AudioFormat::Mp3 => "mp3",
                            AudioFormat::Opus => "opus",
                        };
                        let audio_final2 = audio_output_dir.join(format!("{}.{}", file_stem, audio_ext));
                        let _ = state_tx.send(PipelineState::Muxing);
                        self.muxer
                            .convert_audio(
                                &video_final,
                                &audio_final2,
                                &config.audio_format,
                                &config.audio_quality,
                            )
                            .await?;
                        audio_final2.to_string_lossy().to_string()
                    } else {
                        log::warn!("Both mode: no audio-only stream and no FFmpeg; audio file skipped");
                        String::new()
                    }
                }
            }
        } else if self.muxer.is_available() {
            // Strategy 2: extract audio from video via ffmpeg
            let audio_ext = match config.audio_format {
                AudioFormat::Aac => "m4a",
                AudioFormat::Mp3 => "mp3",
                AudioFormat::Opus => "opus",
            };
            let audio_final = audio_output_dir.join(format!("{}.{}", file_stem, audio_ext));
            let _ = state_tx.send(PipelineState::Muxing);
            self.muxer
                .convert_audio(
                    &video_final,
                    &audio_final,
                    &config.audio_format,
                    &config.audio_quality,
                )
                .await?;
            audio_final.to_string_lossy().to_string()
        } else {
            // Strategy 3: no audio-only stream and no ffmpeg — skip audio
            log::warn!("Both mode: no audio-only stream available and FFmpeg not found; audio file skipped");
            String::new()
        };

        let output = PipelineOutput {
            output_path: video_path_str.clone(),
            video_output_path: Some(video_path_str),
            audio_output_path: if audio_path_str.is_empty() { None } else { Some(audio_path_str) },
        };

        let _ = state_tx.send(PipelineState::Completed { output: output.clone() });
        Ok(output)
    }

    /// Execute download for a single mode (Audio or Video).
    async fn execute_single_mode(
        &self,
        config: &DownloadTaskConfig,
        selected: &SelectedFormats,
        output_dir: &Path,
        final_output: &Path,
        _file_stem: Option<&str>,
        file_stem: &str,
        state_tx: &watch::Sender<PipelineState>,
        cancel_rx: watch::Receiver<bool>,
        http_client: &reqwest::Client,
        download_headers: &HeaderMap,
    ) -> Result<PipelineOutput> {
        if selected.needs_muxing {
            self.download_and_mux(config, selected, output_dir, final_output, state_tx, cancel_rx, http_client, download_headers)
                .await?;
        } else {
            let expose = config.mode == DownloadMode::Video;
            self.download_single_with_retry(&selected.audio, final_output, expose, config, state_tx, cancel_rx, http_client, download_headers)
                .await?;
        }

        // Post-download processing
        let actual_output = if selected.needs_audio_extraction {
            let audio_ext = match config.audio_format {
                AudioFormat::Aac => "m4a",
                AudioFormat::Mp3 => "mp3",
                AudioFormat::Opus => "opus",
            };
            let audio_path = output_dir.join(format!("{}.{}", file_stem, audio_ext));
            if self.muxer.is_available() {
                let _ = state_tx.send(PipelineState::Muxing);
                self.muxer
                    .convert_audio(
                        final_output,
                        &audio_path,
                        &config.audio_format,
                        &config.audio_quality,
                    )
                    .await?;
                let _ = tokio::fs::remove_file(final_output).await;
                audio_path
            } else {
                log::warn!("FFmpeg not available; returning muxed file instead of audio-only");
                final_output.to_path_buf()
            }
        } else if config.mode == DownloadMode::Audio
            && config.audio_format == AudioFormat::Mp3
            && selected.audio.codec != "mp3"
        {
            let mp3_path = output_dir.join(format!("{}.mp3", file_stem));
            if self.muxer.is_available() {
                self.muxer
                    .convert_audio(
                        final_output,
                        &mp3_path,
                        &AudioFormat::Mp3,
                        &config.audio_quality,
                    )
                    .await?;
                let _ = tokio::fs::remove_file(final_output).await;
                mp3_path
            } else {
                final_output.to_path_buf()
            }
        } else {
            final_output.to_path_buf()
        };

        let output_str = actual_output.to_string_lossy().to_string();
        let is_audio = config.mode == DownloadMode::Audio;

        let output = PipelineOutput {
            output_path: output_str.clone(),
            video_output_path: if is_audio { None } else { Some(output_str.clone()) },
            audio_output_path: if is_audio { Some(output_str) } else { None },
        };

        let _ = state_tx.send(PipelineState::Completed { output: output.clone() });
        Ok(output)
    }

    /// Build HTTP headers matching the Innertube client that provided the stream URLs.
    fn build_download_headers(client: &InnertubeClient) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(client.user_agent)
                .unwrap_or_else(|_| HeaderValue::from_static("Mozilla/5.0")),
        );
        if client.is_browser {
            headers.insert(ORIGIN, HeaderValue::from_static("https://www.youtube.com"));
            headers.insert(REFERER, HeaderValue::from_static("https://www.youtube.com/"));
        }
        headers
    }

    /// Returns (player_response, client, po_token_was_used).
    async fn fetch_player_response(
        &self,
        video_id: &str,
        po_token: Option<&str>,
        visitor_data: Option<&str>,
    ) -> Result<(PlayerResponse, &'static InnertubeClient, bool)> {
        use crate::extractor::clients::{WEB, WEB_EMBEDDED, ANDROID, IOS, TV};
        let web_priority: &[&InnertubeClient] = &[&*WEB, &*WEB_EMBEDDED, &*TV, &*ANDROID, &*IOS];

        let clients: &[&InnertubeClient] = if po_token.is_some() {
            web_priority
        } else {
            &*CLIENT_PRIORITY
        };

        for client in clients {
            let (token, vd) = if client.is_browser {
                (po_token, visitor_data)
            } else {
                (None, None)
            };

            let sts = if client.requires_js {
                if let Err(e) = self.ensure_player_js_loaded(video_id).await {
                    log::warn!("Could not load player.js before {} request (no STS): {}", client.name, e);
                }
                self.sig_resolver.lock().sts()
            } else {
                None
            };

            match self.innertube.player(video_id, client, sts, token, vd).await {
                Ok(resp) if resp.is_playable() => {
                    let token_used = token.is_some();
                    log::info!(
                        "Got playable response from {} client for {}{}",
                        client.name,
                        video_id,
                        if token_used { " (with PO token)" } else { "" },
                    );
                    return Ok((resp, client, token_used));
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

        for fmt in &raw_formats {
            if let Some(url) = &fmt.url {
                resolved.push(fmt.to_resolved(url.clone()));
            } else if fmt.signature_cipher.is_some() {
                needs_js = true;
            }
        }

        if needs_js {
            match self.resolve_signature_formats(&raw_formats, video_id).await {
                Ok(sig_formats) => resolved.extend(sig_formats),
                Err(e) => {
                    log::warn!("Signature resolution failed: {}", e);
                }
            }
        } else {
            if let Err(e) = self.ensure_player_js_loaded(video_id).await {
                log::warn!("Failed to load player.js for n-param transformation: {}", e);
            }
        }

        let resolver = self.sig_resolver.lock();
        for fmt in &mut resolved {
            match signatures::apply_n_param(&fmt.url, &resolver) {
                Ok(new_url) => {
                    if new_url != fmt.url {
                        log::debug!("n-param transformed for itag {}", fmt.itag);
                    } else {
                        log::debug!("n-param unchanged for itag {} (no n param or same value)", fmt.itag);
                    }
                    fmt.url = new_url;
                }
                Err(e) => {
                    log::warn!("n-param transformation failed for itag {}: {}", fmt.itag, e);
                }
            }
        }

        Ok(resolved)
    }

    async fn resolve_signature_formats(
        &self,
        raw_formats: &[&StreamFormat],
        video_id: &str,
    ) -> Result<Vec<ResolvedFormat>> {
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
        let html = self.innertube.fetch_watch_page(video_id).await?;
        let player_js_url = extract_player_js_url(&html)?;

        {
            let resolver = self.sig_resolver.lock();
            if resolver.is_loaded_for(&player_js_url) {
                return Ok(());
            }
        }

        let player_js_source = self.innertube.fetch_player_js(&player_js_url).await?;

        let mut resolver = self.sig_resolver.lock();
        resolver.load_player_js(&player_js_url, &player_js_source)?;

        Ok(())
    }

    async fn download_single(
        &self,
        format: &ResolvedFormat,
        output_path: &Path,
        expose_video_path: bool,
        _config: &DownloadTaskConfig,
        state_tx: &watch::Sender<PipelineState>,
        cancel_rx: watch::Receiver<bool>,
        http_client: &reqwest::Client,
        download_headers: &HeaderMap,
    ) -> Result<()> {
        let video_path_for_sse: Option<String> = if expose_video_path {
            Some(output_path.to_string_lossy().into_owned())
        } else {
            None
        };

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
                    video_path: video_path_for_sse.clone(),
                });
            }
        });

        download_with_progress(
            http_client,
            &format.url,
            output_path,
            format.content_length,
            download_headers,
            progress_tx,
            cancel_rx,
        )
        .await?;

        progress_forwarder.abort();
        Ok(())
    }

    /// Like `download_single`, but on a CDN 403 invalidates the player.js cache and retries once
    /// with a freshly transformed n-parameter.
    async fn download_single_with_retry(
        &self,
        format: &ResolvedFormat,
        output_path: &Path,
        expose_video_path: bool,
        config: &DownloadTaskConfig,
        state_tx: &watch::Sender<PipelineState>,
        cancel_rx: watch::Receiver<bool>,
        http_client: &reqwest::Client,
        download_headers: &HeaderMap,
    ) -> Result<()> {
        match self.download_single(format, output_path, expose_video_path, config, state_tx, cancel_rx.clone(), http_client, download_headers).await {
            Ok(()) => Ok(()),
            Err(e) if e.to_string().contains("403") => {
                log::warn!("CDN 403 for itag {} — invalidating player.js cache and retrying", format.itag);
                self.sig_resolver.lock().invalidate();
                if let Err(load_err) = self.ensure_player_js_loaded(&config.video_id).await {
                    log::warn!("Failed to reload player.js for retry: {}", load_err);
                }
                let refreshed_url = {
                    let resolver = self.sig_resolver.lock();
                    signatures::apply_n_param(&format.url, &resolver).unwrap_or_else(|_| format.url.clone())
                };
                let refreshed = ResolvedFormat { url: refreshed_url, ..format.clone() };
                self.download_single(&refreshed, output_path, expose_video_path, config, state_tx, cancel_rx, http_client, download_headers).await
            }
            Err(e) => Err(e),
        }
    }

    /// Fetch a player response from one specific client (no fallback loop).
    async fn fetch_player_response_from_client(
        &self,
        video_id: &str,
        client: &'static InnertubeClient,
        po_token: Option<&str>,
        visitor_data: Option<&str>,
    ) -> Result<(PlayerResponse, &'static InnertubeClient, bool)> {
        let (token, vd) = if client.is_browser {
            (po_token, visitor_data)
        } else {
            (None, None)
        };
        let sts = if client.requires_js {
            if let Err(e) = self.ensure_player_js_loaded(video_id).await {
                log::warn!("Could not load player.js before {} request (no STS): {}", client.name, e);
            }
            self.sig_resolver.lock().sts()
        } else {
            None
        };
        let resp = self.innertube.player(video_id, client, sts, token, vd).await?;
        if !resp.is_playable() {
            let reason = resp.unplayable_reason().unwrap_or_default();
            anyhow::bail!("{} client returned unplayable: {}", client.name, reason);
        }
        let token_used = token.is_some();
        Ok((resp, client, token_used))
    }

    /// Like `download_single_with_retry`, but on a persistent CDN 403 (after n-param refresh)
    /// re-fetches the player response with the WEB client and retries with browser headers.
    /// Returns the original error if the WEB client also fails or has no audio-only stream.
    async fn download_audio_with_client_fallback(
        &self,
        format: &ResolvedFormat,
        output_path: &Path,
        config: &DownloadTaskConfig,
        state_tx: &watch::Sender<PipelineState>,
        cancel_rx: watch::Receiver<bool>,
        http_client: &reqwest::Client,
        download_headers: &HeaderMap,
    ) -> Result<()> {
        let result = self.download_single_with_retry(
            format, output_path, false, config, state_tx, cancel_rx.clone(), http_client, download_headers,
        ).await;

        match result {
            Ok(()) => Ok(()),
            Err(e) => {
                if !e.to_string().contains("403") {
                    return Err(e);
                }
                log::warn!(
                    "Audio CDN 403 persists for itag {} after n-param refresh — retrying with WEB client",
                    format.itag
                );
                match self.fetch_player_response_from_client(
                    &config.video_id,
                    &*WEB,
                    config.po_token.as_deref(),
                    config.visitor_data.as_deref(),
                ).await {
                    Ok((web_response, web_client, web_token_used)) => {
                        let mut web_formats = self.resolve_formats(&web_response, &config.video_id).await?;
                        if web_token_used {
                            if let Some(ref token) = config.po_token {
                                for fmt in &mut web_formats {
                                    if fmt.url.contains('?') {
                                        fmt.url = format!("{}&pot={}", fmt.url, token);
                                    } else {
                                        fmt.url = format!("{}?pot={}", fmt.url, token);
                                    }
                                }
                            }
                        }
                        let web_selected = select_formats(
                            &web_formats,
                            &DownloadMode::Audio,
                            &config.audio_quality,
                            &config.audio_format,
                            None,
                            None,
                            web_token_used,
                        );
                        match web_selected {
                            Ok(s) if !s.needs_audio_extraction => {
                                let web_headers = Self::build_download_headers(web_client);
                                log::info!(
                                    "Retrying audio download with WEB client (itag {})",
                                    s.audio.itag
                                );
                                self.download_single_with_retry(
                                    &s.audio, output_path, false, config, state_tx, cancel_rx,
                                    http_client, &web_headers,
                                ).await
                            }
                            _ => {
                                log::warn!("WEB client has no audio-only adaptive stream");
                                Err(e)
                            }
                        }
                    }
                    Err(web_err) => {
                        log::warn!("WEB client fetch failed for audio retry: {}", web_err);
                        Err(e)
                    }
                }
            }
        }
    }

    async fn download_and_mux(
        &self,
        config: &DownloadTaskConfig,
        selected: &SelectedFormats,
        output_dir: &Path,
        final_output: &Path,
        state_tx: &watch::Sender<PipelineState>,
        cancel_rx: watch::Receiver<bool>,
        http_client: &reqwest::Client,
        download_headers: &HeaderMap,
    ) -> Result<()> {
        let file_stem = &config.video_id;
        let video_format = selected.video.as_ref().unwrap();
        let audio_format = &selected.audio;

        let video_tmp = output_dir.join(format!("{}.video.tmp.{}", file_stem, video_format.container));
        let audio_tmp = output_dir.join(format!("{}.audio.tmp.{}", file_stem, audio_format.container));

        // Download video (expose path so the frontend can stream it while it downloads)
        self.download_single_with_retry(video_format, &video_tmp, true, config, state_tx, cancel_rx.clone(), http_client, download_headers)
            .await?;

        if *cancel_rx.borrow() {
            let _ = tokio::fs::remove_file(&video_tmp).await;
            return Err(YtDlpError::Cancelled.into());
        }

        // Download audio (not a video stream, don't expose path)
        self.download_single_with_retry(audio_format, &audio_tmp, false, config, state_tx, cancel_rx, http_client, download_headers)
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
            log::warn!("FFmpeg not available, output will be video-only");
            tokio::fs::rename(&video_tmp, final_output).await?;
        }

        // Clean up temp files
        let _ = tokio::fs::remove_file(&video_tmp).await;
        let _ = tokio::fs::remove_file(&audio_tmp).await;

        Ok(())
    }
}
