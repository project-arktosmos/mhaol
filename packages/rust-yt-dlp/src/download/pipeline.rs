use anyhow::Result;
use parking_lot::Mutex;
use reqwest::header::{HeaderMap, HeaderValue, ORIGIN, REFERER, USER_AGENT};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::watch;

use crate::download::format::{select_formats, SelectedFormats};
use crate::download::http::{download_with_progress, DownloadProgressUpdate};
use crate::download::muxer::FfmpegMuxer;
use crate::error::YtDlpError;
use crate::extractor::clients::{InnertubeClient, CLIENT_PRIORITY};
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
    pub po_token: Option<String>,
    pub visitor_data: Option<String>,
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

    /// Execute the full download pipeline.
    pub async fn execute(
        &self,
        config: &DownloadTaskConfig,
        state_tx: watch::Sender<PipelineState>,
        cancel_rx: watch::Receiver<bool>,
    ) -> Result<String> {
        let _ = state_tx.send(PipelineState::Fetching);

        // Step 1: Get player response (also returns which client succeeded and whether
        // the PO token was actually used for that client).
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

        // Step 2: Resolve format URLs (handle signatures if needed)
        let mut resolved_formats = self
            .resolve_formats(&player_response, &config.video_id)
            .await?;

        if resolved_formats.is_empty() {
            return Err(YtDlpError::NoSuitableFormat.into());
        }

        // Step 2.5: Append pot=TOKEN to all stream URLs — but ONLY when the PO token
        // was actually used for the successful client request.  When we fell back to
        // ANDROID (a non-browser client), the BotGuard token is meaningless and would
        // just be ignored (or worse, cause an error) on the CDN.
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

        // Step 3: Select formats.
        // Only use adaptive formats when the PO token was actually sent to the client
        // that returned the response. Otherwise, adaptive streams will 403.
        let has_po_token = po_token_was_used;
        let selected = select_formats(
            &resolved_formats,
            &config.mode,
            &config.audio_quality,
            &config.audio_format,
            config.video_quality.as_ref(),
            config.video_format.as_ref(),
            has_po_token,
        )?;

        // Step 4: Build download headers matching the client that provided the stream URLs.
        // Uses the same HTTP client as Innertube (shares cookie store) with per-request headers.
        let download_headers = Self::build_download_headers(client);
        let http_client = self.innertube.http_client();

        let file_stem = &config.video_id;
        let output_dir = Path::new(&config.output_dir);
        let final_output = output_dir.join(format!("{}.{}", file_stem, selected.output_extension));

        // Check cancellation
        if *cancel_rx.borrow() {
            return Err(YtDlpError::Cancelled.into());
        }

        if selected.needs_muxing {
            self.download_and_mux(config, &selected, output_dir, &final_output, &state_tx, cancel_rx, http_client, &download_headers)
                .await?;
        } else {
            self.download_single(&selected.audio, &final_output, config, &state_tx, cancel_rx, http_client, &download_headers)
                .await?;
        }

        // Step 5: Handle post-download processing (audio extraction or format conversion).
        let actual_output = if selected.needs_audio_extraction {
            // We downloaded a muxed (video+audio) container; extract the audio track.
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
                        &final_output,
                        &audio_path,
                        &config.audio_format,
                        &config.audio_quality,
                    )
                    .await?;
                let _ = tokio::fs::remove_file(&final_output).await;
                audio_path
            } else {
                log::warn!("FFmpeg not available; returning muxed file instead of audio-only");
                final_output
            }
        } else if config.mode == DownloadMode::Audio
            && config.audio_format == AudioFormat::Mp3
            && selected.audio.codec != "mp3"
        {
            // Convert adaptive AAC/Opus → MP3
            let mp3_path = output_dir.join(format!("{}.mp3", file_stem));
            if self.muxer.is_available() {
                self.muxer
                    .convert_audio(
                        &final_output,
                        &mp3_path,
                        &AudioFormat::Mp3,
                        &config.audio_quality,
                    )
                    .await?;
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

    /// Build HTTP headers matching the Innertube client that provided the stream URLs.
    ///
    /// Browser clients (WEB, TV, etc.) need Origin/Referer on CDN requests.
    /// Native app clients (Android, iOS) must NOT send them — YouTube detects
    /// the mismatch between an app user-agent and browser-style headers as a bot.
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
    /// `po_token_was_used` is true only when the successful client is a browser client
    /// that actually received the PO token — i.e. adaptive streams can be downloaded
    /// with `pot=TOKEN`. When an app client (ANDROID) succeeds, this is false because
    /// BotGuard tokens don't work with non-browser clients.
    async fn fetch_player_response(
        &self,
        video_id: &str,
        po_token: Option<&str>,
        visitor_data: Option<&str>,
    ) -> Result<(PlayerResponse, &'static InnertubeClient, bool)> {
        // When a BotGuard PO token is available, prefer WEB clients (the token is
        // generated via BotGuard which is web-only; it won't work with ANDROID).
        use crate::extractor::clients::{WEB, WEB_EMBEDDED, ANDROID, IOS, TV};
        let web_priority: &[&InnertubeClient] = &[&*WEB, &*WEB_EMBEDDED, &*TV, &*ANDROID, &*IOS];

        let clients: &[&InnertubeClient] = if po_token.is_some() {
            web_priority
        } else {
            &*CLIENT_PRIORITY
        };

        for client in clients {
            // Only pass PO token + visitorData to browser-based clients
            let (token, vd) = if client.is_browser {
                (po_token, visitor_data)
            } else {
                (None, None)
            };

            match self.innertube.player(video_id, client, token, vd).await {
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

        // First pass: collect formats with direct URLs
        for fmt in &raw_formats {
            if let Some(url) = &fmt.url {
                resolved.push(fmt.to_resolved(url.clone()));
            } else if fmt.signature_cipher.is_some() {
                needs_js = true;
            }
        }

        // Second pass: handle signature ciphers and always load player.js for n-param.
        // Player.js must be loaded even when all formats have direct URLs (e.g. Android client),
        // because the n-parameter in every stream URL must be transformed to avoid throttling.
        if needs_js {
            match self.resolve_signature_formats(&raw_formats, video_id).await {
                Ok(sig_formats) => resolved.extend(sig_formats),
                Err(e) => {
                    log::warn!("Signature resolution failed: {}", e);
                    // Continue with whatever formats we already have
                }
            }
        } else {
            // No signature ciphers, but still need player.js for n-parameter transformation.
            if let Err(e) = self.ensure_player_js_loaded(video_id).await {
                log::warn!("Failed to load player.js for n-param transformation: {}", e);
            }
        }

        // Apply n-parameter transformation to all resolved URLs
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
        http_client: &reqwest::Client,
        download_headers: &HeaderMap,
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
            http_client,
            &video_format.url,
            &video_tmp,
            video_format.content_length,
            download_headers,
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
            http_client,
            &audio_format.url,
            &audio_tmp,
            audio_format.content_length,
            download_headers,
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
