use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;
use tokio::sync::{broadcast, watch};

use crate::config::YtDownloadConfig;
use crate::download::muxer::FfmpegMuxer;
use crate::download::pipeline::{DownloadPipeline, DownloadTaskConfig, PipelineState};
use crate::extractor::innertube::InnertubeApi;
use crate::extractor::playlist;
use crate::extractor::signatures;
use crate::types::*;
use crate::util::extract_video_id;

/// SSE event types broadcast to connected clients.
#[derive(Debug, Clone)]
pub enum SseEvent {
    Progress(DownloadProgress),
    Stats(ManagerStats),
    Connected,
}

/// Shared state accessible by spawned download tasks.
struct SharedState {
    downloads: RwLock<HashMap<String, DownloadProgress>>,
    active_tasks: RwLock<HashMap<String, tokio::task::JoinHandle<()>>>,
    cancel_senders: RwLock<HashMap<String, watch::Sender<bool>>>,
    event_sender: broadcast::Sender<SseEvent>,
}

/// Manages the download queue, active downloads, and state.
pub struct DownloadManager {
    config: RwLock<YtDownloadConfig>,
    shared: Arc<SharedState>,
    pipeline: Arc<DownloadPipeline>,
    innertube: Arc<InnertubeApi>,
}

impl DownloadManager {
    pub fn new(config: YtDownloadConfig) -> Self {
        let innertube = Arc::new(InnertubeApi::new());
        let sig_resolver = signatures::shared_resolver();
        let muxer = Arc::new(FfmpegMuxer::new());
        let pipeline = Arc::new(DownloadPipeline::new(
            innertube.clone(),
            sig_resolver,
            muxer,
        ));

        let (event_sender, _) = broadcast::channel(256);

        let shared = Arc::new(SharedState {
            downloads: RwLock::new(HashMap::new()),
            active_tasks: RwLock::new(HashMap::new()),
            cancel_senders: RwLock::new(HashMap::new()),
            event_sender,
        });

        Self {
            config: RwLock::new(config),
            shared,
            pipeline,
            innertube,
        }
    }

    /// Queue a single download.
    pub fn queue_download(&self, request: QueueDownloadRequest) -> String {
        let config = self.config.read();
        let download_id = uuid::Uuid::new_v4().to_string();

        let mode = request.mode.unwrap_or(DownloadMode::Audio);
        let quality = request.quality.unwrap_or(config.default_quality.clone());
        let format = request.format.unwrap_or(config.default_format.clone());

        let progress = DownloadProgress {
            download_id: download_id.clone(),
            url: request.url.clone(),
            video_id: request.video_id.clone(),
            title: request.title.clone(),
            state: DownloadState::Pending,
            progress: 0.0,
            downloaded_bytes: 0,
            total_bytes: 0,
            output_path: None,
            error: None,
            mode: mode.clone(),
            quality: quality.clone(),
            format: format.clone(),
            video_quality: request.video_quality.clone(),
            video_format: request.video_format.clone(),
            thumbnail_url: None,
            duration_seconds: None,
        };

        {
            self.shared
                .downloads
                .write()
                .insert(download_id.clone(), progress.clone());
        }

        self.broadcast_progress(&progress);
        self.broadcast_stats();

        let task_config = DownloadTaskConfig {
            video_id: request.video_id,
            title: request.title,
            mode,
            audio_quality: quality,
            audio_format: format,
            video_quality: request.video_quality,
            video_format: request.video_format,
            output_dir: config.output_path.clone(),
            po_token: config.po_token.clone(),
            visitor_data: config.visitor_data.clone(),
        };

        self.spawn_download(download_id.clone(), task_config);
        download_id
    }

    /// Queue multiple downloads from a playlist.
    pub fn queue_playlist(&self, request: QueuePlaylistRequest) -> Vec<String> {
        let config = self.config.read();
        let mode = request.mode.unwrap_or(DownloadMode::Audio);
        let quality = request.quality.unwrap_or(config.default_quality.clone());
        let format = request.format.unwrap_or(config.default_format.clone());
        let output_path = config.output_path.clone();
        let po_token = config.po_token.clone();
        let visitor_data = config.visitor_data.clone();
        drop(config);

        let mut ids = Vec::new();

        for video in request.videos {
            let download_id = uuid::Uuid::new_v4().to_string();

            let progress = DownloadProgress {
                download_id: download_id.clone(),
                url: video.url.clone(),
                video_id: video.video_id.clone(),
                title: video.title.clone(),
                state: DownloadState::Pending,
                progress: 0.0,
                downloaded_bytes: 0,
                total_bytes: 0,
                output_path: None,
                error: None,
                mode: mode.clone(),
                quality: quality.clone(),
                format: format.clone(),
                video_quality: request.video_quality.clone(),
                video_format: request.video_format.clone(),
                thumbnail_url: None,
                duration_seconds: None,
            };

            {
                self.shared
                    .downloads
                    .write()
                    .insert(download_id.clone(), progress.clone());
            }

            self.broadcast_progress(&progress);

            let task_config = DownloadTaskConfig {
                video_id: video.video_id,
                title: video.title,
                mode: mode.clone(),
                audio_quality: quality.clone(),
                audio_format: format.clone(),
                video_quality: request.video_quality.clone(),
                video_format: request.video_format.clone(),
                output_dir: output_path.clone(),
                po_token: po_token.clone(),
                visitor_data: visitor_data.clone(),
            };

            self.spawn_download(download_id.clone(), task_config);
            ids.push(download_id);
        }

        self.broadcast_stats();
        ids
    }

    /// Cancel a download.
    pub fn cancel_download(&self, download_id: &str) -> bool {
        if let Some(sender) = self.shared.cancel_senders.read().get(download_id) {
            let _ = sender.send(true);
        }

        if let Some(handle) = self.shared.active_tasks.write().remove(download_id) {
            handle.abort();
        }

        let mut downloads = self.shared.downloads.write();
        if let Some(progress) = downloads.get_mut(download_id) {
            progress.state = DownloadState::Cancelled;
            let progress_clone = progress.clone();
            drop(downloads);
            self.broadcast_progress(&progress_clone);
            self.broadcast_stats();
            true
        } else {
            false
        }
    }

    /// Clear all completed and failed downloads.
    pub fn clear_completed(&self) {
        let mut downloads = self.shared.downloads.write();
        downloads.retain(|_, p| {
            !matches!(
                p.state,
                DownloadState::Completed | DownloadState::Failed | DownloadState::Cancelled
            )
        });
        drop(downloads);
        self.broadcast_stats();
    }

    /// Clear all queued (pending) downloads.
    pub fn clear_queue(&self) {
        let mut downloads = self.shared.downloads.write();
        let pending_ids: Vec<String> = downloads
            .iter()
            .filter(|(_, p)| p.state == DownloadState::Pending)
            .map(|(id, _)| id.clone())
            .collect();

        for id in &pending_ids {
            downloads.remove(id);
            if let Some(handle) = self.shared.active_tasks.write().remove(id) {
                handle.abort();
            }
        }

        drop(downloads);
        self.broadcast_stats();
    }

    /// Get progress for a specific download.
    pub fn get_progress(&self, download_id: &str) -> Option<DownloadProgress> {
        self.shared.downloads.read().get(download_id).cloned()
    }

    /// Get all download progress.
    pub fn get_all_progress(&self) -> Vec<DownloadProgress> {
        self.shared.downloads.read().values().cloned().collect()
    }

    /// Get manager stats.
    pub fn get_stats(&self) -> ManagerStats {
        let downloads = self.shared.downloads.read();
        compute_stats(&downloads)
    }

    /// Get current config.
    pub fn get_config(&self) -> YtDownloadConfig {
        self.config.read().clone()
    }

    /// Update config from JSON.
    pub fn update_config(&self, updates: serde_json::Value) {
        let mut config = self.config.write();
        if let Some(path) = updates.get("outputPath").and_then(|v| v.as_str()) {
            config.output_path = path.to_string();
        }
        if let Some(token) = updates.get("poToken").and_then(|v| v.as_str()) {
            config.po_token = if token.is_empty() {
                None
            } else {
                Some(token.to_string())
            };
        }
        if let Some(visitor_data) = updates.get("visitorData").and_then(|v| v.as_str()) {
            config.visitor_data = if visitor_data.is_empty() {
                None
            } else {
                Some(visitor_data.to_string())
            };
        }
        if let Some(cookies) = updates.get("cookies").and_then(|v| v.as_str()) {
            config.cookies = if cookies.is_empty() {
                None
            } else {
                Some(cookies.to_string())
            };
        }
    }

    /// Subscribe to SSE events.
    pub fn subscribe_events(&self) -> broadcast::Receiver<SseEvent> {
        self.shared.event_sender.subscribe()
    }

    /// Fetch video info (delegates to pipeline).
    pub async fn fetch_video_info(&self, url: &str) -> anyhow::Result<VideoInfo> {
        let video_id = extract_video_id(url)?;
        let (po_token, visitor_data) = {
            let config = self.config.read();
            (config.po_token.clone(), config.visitor_data.clone())
        };
        self.pipeline
            .fetch_video_info(&video_id, po_token.as_deref(), visitor_data.as_deref())
            .await
    }

    /// Fetch playlist info.
    pub async fn fetch_playlist_info(&self, url: &str) -> anyhow::Result<PlaylistInfo> {
        let playlist_id = crate::util::extract_playlist_id(url)?;
        playlist::extract_playlist(&self.innertube, &playlist_id).await
    }

    // ── Private methods ────────────────────────────────────────────

    fn spawn_download(&self, download_id: String, config: DownloadTaskConfig) {
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let (state_tx, mut state_rx) = watch::channel(PipelineState::Fetching);

        self.shared
            .cancel_senders
            .write()
            .insert(download_id.clone(), cancel_tx);

        let pipeline = self.pipeline.clone();
        let shared = self.shared.clone();
        let dl_id = download_id.clone();

        // State forwarding task
        let shared_fwd = shared.clone();
        let dl_id_fwd = download_id.clone();

        let state_forwarder = tokio::spawn(async move {
            while state_rx.changed().await.is_ok() {
                let state = state_rx.borrow().clone();
                let mut map = shared_fwd.downloads.write();

                if let Some(progress) = map.get_mut(&dl_id_fwd) {
                    match &state {
                        PipelineState::Fetching => {
                            progress.state = DownloadState::Fetching;
                        }
                        PipelineState::Downloading { downloaded, total } => {
                            progress.state = DownloadState::Downloading;
                            progress.downloaded_bytes = *downloaded;
                            progress.total_bytes = *total;
                            progress.progress = if *total > 0 {
                                *downloaded as f64 / *total as f64
                            } else {
                                0.0
                            };
                        }
                        PipelineState::Muxing => {
                            progress.state = DownloadState::Muxing;
                        }
                        PipelineState::Completed { output_path } => {
                            progress.state = DownloadState::Completed;
                            progress.progress = 1.0;
                            progress.output_path = Some(output_path.clone());
                        }
                        PipelineState::Failed { error } => {
                            progress.state = DownloadState::Failed;
                            progress.error = Some(error.clone());
                        }
                    }

                    let progress_clone = progress.clone();
                    drop(map);
                    let _ = shared_fwd
                        .event_sender
                        .send(SseEvent::Progress(progress_clone));
                }
            }
        });

        // Main download task
        let shared_main = shared.clone();

        let handle = tokio::spawn(async move {
            let result = pipeline.execute(&config, state_tx, cancel_rx).await;

            // Update final state
            let mut map = shared_main.downloads.write();

            if let Some(progress) = map.get_mut(&dl_id) {
                match result {
                    Ok(output_path) => {
                        progress.state = DownloadState::Completed;
                        progress.progress = 1.0;
                        progress.output_path = Some(output_path);
                    }
                    Err(e) => {
                        if progress.state != DownloadState::Cancelled {
                            log::error!(
                                "Download {} ({}) failed: {}",
                                &dl_id,
                                progress.title,
                                e
                            );
                            progress.state = DownloadState::Failed;
                            progress.error = Some(e.to_string());
                        }
                    }
                }

                let progress_clone = progress.clone();
                let stats = compute_stats(&map);
                drop(map);

                let _ = shared_main
                    .event_sender
                    .send(SseEvent::Progress(progress_clone));
                let _ = shared_main.event_sender.send(SseEvent::Stats(stats));
            } else {
                drop(map);
            }

            // Cleanup
            state_forwarder.abort();
            shared_main.cancel_senders.write().remove(&dl_id);
            shared_main.active_tasks.write().remove(&dl_id);
        });

        self.shared
            .active_tasks
            .write()
            .insert(download_id, handle);
    }

    fn broadcast_progress(&self, progress: &DownloadProgress) {
        let _ = self
            .shared
            .event_sender
            .send(SseEvent::Progress(progress.clone()));
    }

    fn broadcast_stats(&self) {
        let stats = self.get_stats();
        let _ = self.shared.event_sender.send(SseEvent::Stats(stats));
    }
}

fn compute_stats(downloads: &HashMap<String, DownloadProgress>) -> ManagerStats {
    let mut stats = ManagerStats {
        active_downloads: 0,
        queued_downloads: 0,
        completed_downloads: 0,
        failed_downloads: 0,
        ytdlp_available: true,
        ytdlp_version: Some(format!("native-rust-{}", env!("CARGO_PKG_VERSION"))),
    };

    for progress in downloads.values() {
        match progress.state {
            DownloadState::Pending => stats.queued_downloads += 1,
            DownloadState::Fetching | DownloadState::Downloading | DownloadState::Muxing => {
                stats.active_downloads += 1;
            }
            DownloadState::Completed => stats.completed_downloads += 1,
            DownloadState::Failed | DownloadState::Cancelled => stats.failed_downloads += 1,
        }
    }

    stats
}
