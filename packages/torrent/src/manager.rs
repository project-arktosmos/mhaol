use anyhow::Result;
use librqbit::http_api::HttpApi;
use librqbit::{AddTorrent, AddTorrentOptions, Api, Session, SessionOptions, SessionPersistenceConfig};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use crate::config::{TorrentConfig, DEFAULT_TRACKERS};
use crate::types::{AddTorrentRequest, TorrentFile, TorrentInfo, TorrentState, TorrentStats};
use crate::util::{get_unix_timestamp, parse_magnet_uri};

#[derive(Debug, Clone)]
pub struct TrackingInfo {
    pub output_path: Option<String>,
}

pub struct TorrentManager {
    session: RwLock<Option<Arc<Session>>>,
    config: RwLock<TorrentConfig>,
    http_server_handle: RwLock<Option<JoinHandle<()>>>,
    tracking_info: RwLock<HashMap<String, TrackingInfo>>,
    completed_torrents: RwLock<HashSet<String>>,
    auto_paused: RwLock<Vec<String>>,
}

impl TorrentManager {
    pub fn new() -> Self {
        Self {
            session: RwLock::new(None),
            config: RwLock::new(TorrentConfig::default()),
            http_server_handle: RwLock::new(None),
            tracking_info: RwLock::new(HashMap::new()),
            completed_torrents: RwLock::new(HashSet::new()),
            auto_paused: RwLock::new(Vec::new()),
        }
    }

    pub async fn initialize(&self, config: TorrentConfig) -> Result<()> {
        let download_path = config.download_path.clone();
        let persistence_dir = download_path.join(".rqbit");
        std::fs::create_dir_all(&persistence_dir)?;

        let mut trackers: HashSet<url::Url> = DEFAULT_TRACKERS
            .iter()
            .filter_map(|s| s.parse().ok())
            .collect();

        for tracker in &config.extra_trackers {
            if let Ok(url) = tracker.parse() {
                trackers.insert(url);
            }
        }

        log::info!(
            "Initializing torrent session with {} trackers",
            trackers.len()
        );

        let opts = SessionOptions {
            persistence: Some(SessionPersistenceConfig::Json {
                folder: Some(persistence_dir.clone()),
            }),
            listen_port_range: Some(config.listen_port_range.clone()),
            enable_upnp_port_forwarding: config.enable_upnp,
            fastresume: config.fast_resume,
            trackers: trackers.clone(),
            disable_dht_persistence: config.disable_dht_persistence,
            ..Default::default()
        };

        // Try to initialize with persistence first
        let session = match Session::new_with_opts(download_path.clone(), opts).await {
            Ok(s) => s,
            Err(e) => {
                log::warn!(
                    "Failed to initialize with persistence ({}), clearing state and retrying...",
                    e
                );

                // Clear the corrupted persistence folder
                if persistence_dir.exists() {
                    if let Err(rm_err) = std::fs::remove_dir_all(&persistence_dir) {
                        log::warn!("Failed to remove persistence dir: {}", rm_err);
                    }
                    std::fs::create_dir_all(&persistence_dir)?;
                }

                // Retry with fresh persistence
                let retry_opts = SessionOptions {
                    persistence: Some(SessionPersistenceConfig::Json {
                        folder: Some(persistence_dir),
                    }),
                    listen_port_range: Some(config.listen_port_range.clone()),
                    enable_upnp_port_forwarding: config.enable_upnp,
                    fastresume: config.fast_resume,
                    trackers,
                    disable_dht_persistence: config.disable_dht_persistence,
                    ..Default::default()
                };

                Session::new_with_opts(download_path.clone(), retry_opts).await?
            }
        };

        *self.session.write() = Some(session.clone());
        *self.config.write() = config.clone();

        // Start HTTP API server for peer injection if configured
        if config.http_api_bind_addr.is_some() {
            self.start_http_server(session).await;
        }

        log::info!("Torrent manager initialized");
        Ok(())
    }

    async fn start_http_server(&self, session: Arc<Session>) {
        let bind_addr = {
            let config = self.config.read();
            match &config.http_api_bind_addr {
                Some(addr) => addr.clone(),
                None => return,
            }
        };

        let api = Api::new(session, None, None);
        let http_api = HttpApi::new(api, None);

        let listener = match TcpListener::bind(&bind_addr).await {
            Ok(l) => l,
            Err(e) => {
                log::warn!(
                    "Failed to bind HTTP API server to {}: {}. Peer injection will not work.",
                    bind_addr,
                    e
                );
                return;
            }
        };

        let server_future = http_api.make_http_api_and_run(listener, None);
        let handle = tokio::spawn(async move {
            if let Err(e) = server_future.await {
                log::error!("Torrent HTTP API server error: {}", e);
            }
        });
        *self.http_server_handle.write() = Some(handle);
        log::info!("Torrent HTTP API started on {}", bind_addr);
    }

    pub(crate) fn session(&self) -> Option<Arc<Session>> {
        self.session.read().clone()
    }

    pub(crate) fn api(&self) -> Option<Api> {
        self.session().map(|s| Api::new(s, None, None))
    }

    pub fn download_path(&self) -> PathBuf {
        self.config.read().download_path.clone()
    }

    pub fn set_download_path(&self, path: PathBuf) {
        self.config.write().download_path = path;
    }

    pub fn is_initialized(&self) -> bool {
        self.session.read().is_some()
    }

    pub fn set_tracking_info(&self, info_hash: String, info: TrackingInfo) {
        self.tracking_info.write().insert(info_hash, info);
    }

    pub fn get_tracking_info(&self, info_hash: &str) -> Option<TrackingInfo> {
        self.tracking_info.read().get(info_hash).cloned()
    }

    pub fn remove_tracking_info(&self, info_hash: &str) {
        self.tracking_info.write().remove(info_hash);
        self.completed_torrents.write().remove(info_hash);
    }

    pub fn is_torrent_completed(&self, info_hash: &str) -> bool {
        self.completed_torrents.read().contains(info_hash)
    }

    pub fn mark_torrent_completed(&self, info_hash: String) {
        self.completed_torrents.write().insert(info_hash);
    }

    // ── High-level operations (migrated from commands.rs) ────────────

    pub async fn add(&self, request: AddTorrentRequest) -> Result<TorrentInfo> {
        let api = self
            .api()
            .ok_or_else(|| anyhow::anyhow!("Torrent client not initialized"))?;

        let is_magnet = request.source.starts_with("magnet:");
        let is_url =
            request.source.starts_with("http://") || request.source.starts_with("https://");

        let magnet_info = if is_magnet {
            parse_magnet_uri(&request.source)
        } else {
            None
        };

        let output_path = request
            .download_path
            .clone()
            .unwrap_or_else(|| self.download_path().to_string_lossy().to_string());

        // For .torrent files, load synchronously to get metadata
        if !is_magnet && !is_url {
            let add_torrent = AddTorrent::from_local_filename(&request.source)
                .map_err(|e| anyhow::anyhow!("Failed to load torrent file: {}", e))?;

            let options = AddTorrentOptions {
                overwrite: true,
                output_folder: request.download_path.clone(),
                ..Default::default()
            };

            let response = api
                .api_add_torrent(add_torrent, Some(options))
                .await
                .map_err(|e| anyhow::anyhow!("Failed to add torrent: {}", e))?;

            let id = response.id.unwrap_or(0);
            let name = response
                .details
                .name
                .clone()
                .unwrap_or_else(|| "Unknown".to_string());
            let info_hash = response.details.info_hash.to_string();
            let size = response
                .details
                .stats
                .as_ref()
                .map(|s| s.total_bytes)
                .unwrap_or(0);

            let file_path = format!("{}/{}", output_path, name);

            self.set_tracking_info(
                info_hash.clone(),
                TrackingInfo {
                    output_path: Some(file_path.clone()),
                },
            );

            return Ok(TorrentInfo {
                id,
                name,
                info_hash,
                size,
                progress: 0.0,
                download_speed: 0,
                upload_speed: 0,
                peers: 0,
                seeds: 0,
                state: TorrentState::Initializing,
                added_at: get_unix_timestamp(),
                eta: None,
                output_path: Some(file_path),
            });
        }

        // For magnet/URL: spawn the add operation in background and return immediately
        let (info_hash, name) =
            magnet_info.ok_or_else(|| anyhow::anyhow!("Could not parse magnet URI"))?;

        log::info!(
            "[torrent_add] Magnet parsed: info_hash={}, name={}",
            info_hash,
            name
        );

        let file_path = format!("{}/{}", output_path, name);

        self.set_tracking_info(
            info_hash.clone(),
            TrackingInfo {
                output_path: Some(file_path.clone()),
            },
        );

        // Spawn the actual torrent add in background
        let source = request.source.clone();
        let download_path = request.download_path.clone();
        let api_clone = api.clone();

        tokio::spawn(async move {
            let add_torrent = AddTorrent::from_url(&source);
            let options = AddTorrentOptions {
                overwrite: true,
                output_folder: download_path,
                ..Default::default()
            };

            match api_clone
                .api_add_torrent(add_torrent, Some(options))
                .await
            {
                Ok(response) => {
                    log::info!(
                        "[torrent_add background] Torrent metadata resolved: id={:?}, name={:?}",
                        response.id,
                        response.details.name
                    );
                }
                Err(e) => {
                    log::error!("[torrent_add background] Failed to add torrent: {}", e);
                }
            }
        });

        log::info!("[torrent_add] Returning immediately, background task spawned");

        Ok(TorrentInfo {
            id: 0,
            name,
            info_hash,
            size: 0,
            progress: 0.0,
            download_speed: 0,
            upload_speed: 0,
            peers: 0,
            seeds: 0,
            state: TorrentState::Initializing,
            added_at: get_unix_timestamp(),
            eta: None,
            output_path: Some(file_path),
        })
    }

    pub async fn list(&self) -> Result<Vec<TorrentInfo>> {
        let api = self
            .api()
            .ok_or_else(|| anyhow::anyhow!("Torrent client not initialized"))?;

        let list = api.api_torrent_list();
        let mut torrents = Vec::new();

        for item in list.torrents {
            let id = match item.id {
                Some(id) => id,
                None => continue,
            };

            let (progress, download_speed, upload_speed, peers, seeds, torrent_state, eta, size) =
                if let Ok(handle) = api.mgr_handle(id.into()) {
                    let stats = handle.stats();

                    let total_bytes = stats.total_bytes;
                    let progress_bytes = stats.progress_bytes;
                    let is_finished = stats.finished;

                    let progress = if is_finished {
                        1.0
                    } else if total_bytes > 0 {
                        progress_bytes as f64 / total_bytes as f64
                    } else {
                        0.0
                    };

                    let internal_state = handle.with_state(|s| s.name().to_string());
                    let torrent_state = match internal_state.as_str() {
                        "live" => {
                            if is_finished {
                                TorrentState::Seeding
                            } else {
                                TorrentState::Downloading
                            }
                        }
                        "paused" => TorrentState::Paused,
                        "error" => TorrentState::Error,
                        _ => TorrentState::Initializing,
                    };

                    let (dl_speed, ul_speed, p, s) = if let Some(live) = &stats.live {
                        (
                            (live.download_speed.mbps * 1_000_000.0 / 8.0) as u64,
                            (live.upload_speed.mbps * 1_000_000.0 / 8.0) as u64,
                            live.snapshot.peer_stats.live as u32,
                            live.snapshot.peer_stats.seen as u32,
                        )
                    } else {
                        (0, 0, 0, 0)
                    };

                    let eta_val = if dl_speed > 0 && progress < 1.0 {
                        let remaining = total_bytes.saturating_sub(progress_bytes);
                        Some(remaining / dl_speed)
                    } else {
                        None
                    };

                    (
                        progress,
                        dl_speed,
                        ul_speed,
                        p,
                        s,
                        torrent_state,
                        eta_val,
                        total_bytes,
                    )
                } else {
                    (0.0, 0, 0, 0, 0, TorrentState::Initializing, None, 0)
                };

            let info_hash = item.info_hash.to_string();
            let name = item.name.unwrap_or_else(|| "Unknown".to_string());

            // Look up tracking info by info_hash, create if missing
            let tracking = match self.get_tracking_info(&info_hash) {
                Some(t) => t,
                None => {
                    let output_path = self.download_path().to_string_lossy().to_string();
                    let file_path = format!("{}/{}", output_path, name);
                    let new_tracking = TrackingInfo {
                        output_path: Some(file_path),
                    };
                    self.set_tracking_info(info_hash.clone(), new_tracking.clone());
                    new_tracking
                }
            };

            torrents.push(TorrentInfo {
                id,
                name,
                info_hash,
                size,
                progress,
                download_speed,
                upload_speed,
                peers,
                seeds,
                state: torrent_state,
                added_at: get_unix_timestamp(),
                eta,
                output_path: tracking.output_path,
            });
        }

        Ok(torrents)
    }

    pub async fn pause(&self, id: usize) -> Result<()> {
        let api = self
            .api()
            .ok_or_else(|| anyhow::anyhow!("Torrent client not initialized"))?;

        api.api_torrent_action_pause(id.into())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to pause torrent: {}", e))?;

        Ok(())
    }

    pub async fn resume(&self, id: usize) -> Result<()> {
        let api = self
            .api()
            .ok_or_else(|| anyhow::anyhow!("Torrent client not initialized"))?;

        api.api_torrent_action_start(id.into())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to resume torrent: {}", e))?;

        Ok(())
    }

    pub async fn remove(&self, id: usize) -> Result<()> {
        let api = self
            .api()
            .ok_or_else(|| anyhow::anyhow!("Torrent client not initialized"))?;

        api.api_torrent_action_delete(id.into())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to remove torrent: {}", e))?;

        Ok(())
    }

    pub async fn stats(&self) -> Result<TorrentStats> {
        let api = self
            .api()
            .ok_or_else(|| anyhow::anyhow!("Torrent client not initialized"))?;

        let list = api.api_torrent_list();
        let mut total_download = 0u64;
        let mut total_upload = 0u64;
        let mut download_speed = 0u64;
        let mut upload_speed = 0u64;
        let mut active = 0u32;

        for item in list.torrents {
            let id = match item.id {
                Some(id) => id,
                None => continue,
            };

            if let Ok(details) = api.api_torrent_details(id.into()) {
                if let Some(stats) = &details.stats {
                    total_download += stats.progress_bytes;
                    total_upload += stats.uploaded_bytes;

                    if let Some(live) = &stats.live {
                        download_speed += (live.download_speed.mbps * 1_000_000.0 / 8.0) as u64;
                        upload_speed += (live.upload_speed.mbps * 1_000_000.0 / 8.0) as u64;
                    }

                    if !stats.finished {
                        active += 1;
                    }
                }
            }
        }

        Ok(TorrentStats {
            total_downloaded: total_download,
            total_uploaded: total_upload,
            download_speed,
            upload_speed,
            active_torrents: active,
        })
    }

    pub async fn remove_all(&self) -> Result<u32> {
        let api = match self.api() {
            Some(a) => a,
            None => return Ok(0),
        };

        let list = api.api_torrent_list();
        let mut removed = 0u32;

        for item in list.torrents {
            if let Some(id) = item.id {
                match api.api_torrent_action_delete(id.into()).await {
                    Ok(_) => {
                        removed += 1;
                        log::info!("Removed torrent id={}", id);
                    }
                    Err(e) => {
                        log::warn!("Failed to remove torrent id={}: {}", id, e);
                    }
                }
            }
        }

        log::info!("Removed {} torrents from client", removed);
        Ok(removed)
    }

    pub async fn clear_storage(&self) -> Result<()> {
        let download_path = self.download_path();

        if download_path.as_os_str().is_empty() {
            log::info!("No download path set, skipping storage clear");
            return Ok(());
        }

        // Clear the .rqbit persistence folder
        let persistence_dir = download_path.join(".rqbit");
        if persistence_dir.exists() {
            log::info!("Deleting persistence directory: {:?}", persistence_dir);
            std::fs::remove_dir_all(&persistence_dir)
                .map_err(|e| anyhow::anyhow!("Failed to delete .rqbit folder: {}", e))?;
        }

        // Clear all contents of the download directory (but keep the directory itself)
        if download_path.exists() {
            log::info!("Clearing download directory: {:?}", download_path);
            for entry in std::fs::read_dir(&download_path)
                .map_err(|e| anyhow::anyhow!("Failed to read download directory: {}", e))?
            {
                let entry = entry
                    .map_err(|e| anyhow::anyhow!("Failed to read directory entry: {}", e))?;
                let path = entry.path();

                if path.is_dir() {
                    std::fs::remove_dir_all(&path).map_err(|e| {
                        anyhow::anyhow!("Failed to delete directory {:?}: {}", path, e)
                    })?;
                } else {
                    std::fs::remove_file(&path).map_err(|e| {
                        anyhow::anyhow!("Failed to delete file {:?}: {}", path, e)
                    })?;
                }
            }
            log::info!("Download directory cleared");
        }

        Ok(())
    }

    pub async fn debug_info(&self) -> Result<Vec<String>> {
        let mut logs: Vec<String> = Vec::new();

        logs.push(format!(
            "[{}] === TORRENT DEBUG INFO ===",
            get_unix_timestamp()
        ));

        let session = match self.session() {
            Some(s) => {
                logs.push("Session is initialized".to_string());
                s
            }
            None => {
                logs.push("Session is NOT initialized".to_string());
                return Ok(logs);
            }
        };

        let api = match self.api() {
            Some(a) => {
                logs.push("API is available".to_string());
                a
            }
            None => {
                logs.push("API is NOT available".to_string());
                return Ok(logs);
            }
        };

        logs.push("\n=== SESSION INFO ===".to_string());

        match session.tcp_listen_port() {
            Some(port) => logs.push(format!(
                "TCP Listen Port: {} (listening for incoming connections)",
                port
            )),
            None => logs.push(
                "TCP Listen Port: NONE (cannot receive incoming connections!)".to_string(),
            ),
        };

        let session_stats = api.api_session_stats();
        logs.push("Session Stats:".to_string());
        logs.push(format!(
            "  download_speed: {} bytes/sec",
            session_stats.download_speed
        ));
        logs.push(format!(
            "  upload_speed: {} bytes/sec",
            session_stats.upload_speed
        ));
        logs.push(format!(
            "  fetched_bytes: {}",
            session_stats.fetched_bytes
        ));
        logs.push(format!(
            "  uploaded_bytes: {}",
            session_stats.uploaded_bytes
        ));

        match api.api_dht_stats() {
            Ok(dht_stats) => {
                logs.push("DHT Stats:".to_string());
                logs.push(format!("  {:?}", dht_stats));
            }
            Err(e) => {
                logs.push(format!("DHT Stats: ERROR - {}", e));
            }
        };

        logs.push("\n=== TORRENTS ===".to_string());
        let list = api.api_torrent_list();
        logs.push(format!("Torrent count: {}", list.torrents.len()));

        for (idx, item) in list.torrents.iter().enumerate() {
            logs.push(format!("\n--- Torrent #{} ---", idx));
            logs.push(format!("  ID: {:?}", item.id));
            logs.push(format!("  Name: {:?}", item.name));
            logs.push(format!("  Info hash: {:?}", item.info_hash));

            if let Some(id) = item.id {
                if let Ok(handle) = api.mgr_handle(id.into()) {
                    let state_name = handle.with_state(|s| s.name().to_string());
                    logs.push(format!("  Internal State: {}", state_name));

                    let handle_stats = handle.stats();
                    logs.push("  Handle Stats:".to_string());
                    logs.push(format!("    state: {:?}", handle_stats.state));
                    logs.push(format!("    error: {:?}", handle_stats.error));
                    logs.push(format!(
                        "    progress_bytes: {}",
                        handle_stats.progress_bytes
                    ));
                    logs.push(format!("    total_bytes: {}", handle_stats.total_bytes));
                    logs.push(format!("    finished: {}", handle_stats.finished));

                    if let Some(live) = &handle_stats.live {
                        logs.push(format!(
                            "    Live: download={} Mbps, upload={} Mbps",
                            live.download_speed.mbps, live.upload_speed.mbps
                        ));
                        logs.push(format!(
                            "    Peers: live={}, connecting={}, seen={}, dead={}",
                            live.snapshot.peer_stats.live,
                            live.snapshot.peer_stats.connecting,
                            live.snapshot.peer_stats.seen,
                            live.snapshot.peer_stats.dead
                        ));
                    }
                }

                match api.api_torrent_details(id.into()) {
                    Ok(details) => {
                        logs.push("  Details: OK".to_string());
                        logs.push(format!(
                            "    files: {:?}",
                            details.files.as_ref().map(|f| f.len())
                        ));

                        if let Some(stats) = &details.stats {
                            logs.push(format!("    total_bytes: {}", stats.total_bytes));
                            logs.push(format!("    progress_bytes: {}", stats.progress_bytes));
                            logs.push(format!("    uploaded_bytes: {}", stats.uploaded_bytes));
                            logs.push(format!("    finished: {}", stats.finished));

                            if let Some(live) = &stats.live {
                                logs.push("    Live stats: YES".to_string());
                                logs.push(format!(
                                    "      download_speed: {} Mbps",
                                    live.download_speed.mbps
                                ));
                                logs.push(format!(
                                    "      upload_speed: {} Mbps",
                                    live.upload_speed.mbps
                                ));
                                logs.push(format!(
                                    "      peers_live: {}",
                                    live.snapshot.peer_stats.live
                                ));
                            } else {
                                logs.push(
                                    "    Live stats: NO (torrent may be paused or not started)"
                                        .to_string(),
                                );
                            }
                        } else {
                            logs.push(
                                "    Stats: NONE (metadata not yet fetched?)".to_string(),
                            );
                        }
                    }
                    Err(e) => {
                        logs.push(format!("  Details ERROR: {}", e));
                    }
                }
            }
        }

        logs.push(format!(
            "\n[{}] === END DEBUG INFO ===",
            get_unix_timestamp()
        ));

        Ok(logs)
    }

    pub fn get_http_api_addr(&self) -> Option<String> {
        self.config.read().http_api_bind_addr.clone()
    }

    pub async fn list_files(&self, torrent_id: usize) -> Result<Vec<TorrentFile>> {
        let api = self
            .api()
            .ok_or_else(|| anyhow::anyhow!("Torrent client not initialized"))?;

        let details = api
            .api_torrent_details(torrent_id.into())
            .map_err(|e| anyhow::anyhow!("Failed to get torrent details: {}", e))?;

        let files = details
            .files
            .unwrap_or_default()
            .into_iter()
            .enumerate()
            .map(|(idx, f)| TorrentFile {
                id: idx,
                name: f.name,
                size: f.length,
            })
            .collect();

        Ok(files)
    }

    pub async fn pause_all_except(&self, info_hash: &str) -> Result<()> {
        let torrents = self.list().await?;
        let mut auto_paused = Vec::new();

        for t in &torrents {
            if t.info_hash != info_hash && t.state == TorrentState::Downloading {
                if let Err(e) = self.pause(t.id).await {
                    log::warn!("Failed to auto-pause torrent {}: {}", t.info_hash, e);
                } else {
                    auto_paused.push(t.info_hash.clone());
                }
            }
        }

        log::info!(
            "Auto-paused {} torrents for streaming {}",
            auto_paused.len(),
            info_hash
        );
        *self.auto_paused.write() = auto_paused;
        Ok(())
    }

    pub async fn resume_auto_paused(&self) -> Result<()> {
        let to_resume: Vec<String> = self.auto_paused.write().drain(..).collect();

        if to_resume.is_empty() {
            return Ok(());
        }

        let torrents = self.list().await?;
        for info_hash in &to_resume {
            if let Some(t) = torrents.iter().find(|t| &t.info_hash == info_hash) {
                if t.state == TorrentState::Paused {
                    if let Err(e) = self.resume(t.id).await {
                        log::warn!("Failed to resume torrent {}: {}", info_hash, e);
                    }
                }
            }
        }

        log::info!("Resumed {} auto-paused torrents", to_resume.len());
        Ok(())
    }

    pub fn complete_download(&self, info_hash: String, output_path: String) -> Result<()> {
        let path = std::path::Path::new(&output_path);

        // Compute size for informational purposes
        let _size_bytes = if path.is_dir() {
            fn dir_size(path: &std::path::Path) -> std::io::Result<u64> {
                let mut size = 0;
                for entry in std::fs::read_dir(path)? {
                    let entry = entry?;
                    let metadata = entry.metadata()?;
                    if metadata.is_dir() {
                        size += dir_size(&entry.path())?;
                    } else {
                        size += metadata.len();
                    }
                }
                Ok(size)
            }
            dir_size(path).ok()
        } else {
            std::fs::metadata(&output_path).map(|m| m.len()).ok()
        };

        self.mark_torrent_completed(info_hash);

        Ok(())
    }
}

impl Default for TorrentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TorrentConfig;
    use std::fs;
    use tempfile::TempDir;

    // ── Construction ────────────────────────────────────────────────

    #[test]
    fn new_creates_uninitialized_manager() {
        let mgr = TorrentManager::new();
        assert!(!mgr.is_initialized());
    }

    #[test]
    fn default_creates_uninitialized_manager() {
        let mgr = TorrentManager::default();
        assert!(!mgr.is_initialized());
    }

    #[test]
    fn new_download_path_is_empty() {
        let mgr = TorrentManager::new();
        assert_eq!(mgr.download_path(), PathBuf::new());
    }

    // ── Tracking info ───────────────────────────────────────────────

    #[test]
    fn set_and_get_tracking_info() {
        let mgr = TorrentManager::new();
        let info = TrackingInfo {
            output_path: Some("/tmp/test".to_string()),
        };
        mgr.set_tracking_info("abc123".to_string(), info);
        let retrieved = mgr.get_tracking_info("abc123");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().output_path, Some("/tmp/test".to_string()));
    }

    #[test]
    fn get_tracking_info_nonexistent() {
        let mgr = TorrentManager::new();
        assert!(mgr.get_tracking_info("nonexistent").is_none());
    }

    #[test]
    fn set_tracking_info_overwrites() {
        let mgr = TorrentManager::new();
        mgr.set_tracking_info(
            "abc".to_string(),
            TrackingInfo {
                output_path: Some("/first".to_string()),
            },
        );
        mgr.set_tracking_info(
            "abc".to_string(),
            TrackingInfo {
                output_path: Some("/second".to_string()),
            },
        );
        let retrieved = mgr.get_tracking_info("abc").unwrap();
        assert_eq!(retrieved.output_path, Some("/second".to_string()));
    }

    #[test]
    fn remove_tracking_info() {
        let mgr = TorrentManager::new();
        mgr.set_tracking_info(
            "abc".to_string(),
            TrackingInfo {
                output_path: None,
            },
        );
        mgr.mark_torrent_completed("abc".to_string());
        assert!(mgr.get_tracking_info("abc").is_some());
        assert!(mgr.is_torrent_completed("abc"));

        mgr.remove_tracking_info("abc");
        assert!(mgr.get_tracking_info("abc").is_none());
        assert!(!mgr.is_torrent_completed("abc"));
    }

    #[test]
    fn remove_tracking_info_nonexistent_is_noop() {
        let mgr = TorrentManager::new();
        mgr.remove_tracking_info("nonexistent"); // should not panic
    }

    // ── Torrent completion tracking ─────────────────────────────────

    #[test]
    fn mark_and_check_torrent_completed() {
        let mgr = TorrentManager::new();
        assert!(!mgr.is_torrent_completed("hash1"));
        mgr.mark_torrent_completed("hash1".to_string());
        assert!(mgr.is_torrent_completed("hash1"));
    }

    #[test]
    fn mark_completed_idempotent() {
        let mgr = TorrentManager::new();
        mgr.mark_torrent_completed("hash1".to_string());
        mgr.mark_torrent_completed("hash1".to_string());
        assert!(mgr.is_torrent_completed("hash1"));
    }

    #[test]
    fn multiple_completed_torrents() {
        let mgr = TorrentManager::new();
        mgr.mark_torrent_completed("a".to_string());
        mgr.mark_torrent_completed("b".to_string());
        mgr.mark_torrent_completed("c".to_string());
        assert!(mgr.is_torrent_completed("a"));
        assert!(mgr.is_torrent_completed("b"));
        assert!(mgr.is_torrent_completed("c"));
        assert!(!mgr.is_torrent_completed("d"));
    }

    // ── complete_download (filesystem) ──────────────────────────────

    #[test]
    fn complete_download_with_file() {
        let tmp = TempDir::new().unwrap();
        let file_path = tmp.path().join("test.txt");
        fs::write(&file_path, "hello world").unwrap();

        let mgr = TorrentManager::new();
        let result =
            mgr.complete_download("hash1".to_string(), file_path.to_string_lossy().to_string());
        assert!(result.is_ok());
        assert!(mgr.is_torrent_completed("hash1"));
    }

    #[test]
    fn complete_download_with_directory() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("subdir");
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("a.txt"), "aaa").unwrap();
        fs::write(dir.join("b.txt"), "bbbbb").unwrap();

        // Nested dir
        let nested = dir.join("nested");
        fs::create_dir(&nested).unwrap();
        fs::write(nested.join("c.txt"), "cc").unwrap();

        let mgr = TorrentManager::new();
        let result =
            mgr.complete_download("hash2".to_string(), dir.to_string_lossy().to_string());
        assert!(result.is_ok());
        assert!(mgr.is_torrent_completed("hash2"));
    }

    #[test]
    fn complete_download_nonexistent_path() {
        let mgr = TorrentManager::new();
        // Non-existent path — should still succeed (size computation returns None, but no error)
        let result =
            mgr.complete_download("hash3".to_string(), "/nonexistent/path/file.bin".to_string());
        assert!(result.is_ok());
        assert!(mgr.is_torrent_completed("hash3"));
    }

    // ── clear_storage ───────────────────────────────────────────────

    #[tokio::test]
    async fn clear_storage_empty_path_is_noop() {
        let mgr = TorrentManager::new();
        // default download_path is empty
        let result = mgr.clear_storage().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn clear_storage_removes_rqbit_dir_and_contents() {
        let tmp = TempDir::new().unwrap();
        let download_path = tmp.path().to_path_buf();

        // Set up manager config with this path
        let mgr = TorrentManager::new();
        {
            let mut config = mgr.config.write();
            config.download_path = download_path.clone();
        }

        // Create .rqbit dir and some files
        let rqbit_dir = download_path.join(".rqbit");
        fs::create_dir_all(&rqbit_dir).unwrap();
        fs::write(rqbit_dir.join("state.json"), "{}").unwrap();

        // Create some download files
        fs::write(download_path.join("file1.dat"), "data1").unwrap();
        let subdir = download_path.join("album");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("track.mp3"), "audio").unwrap();

        let result = mgr.clear_storage().await;
        assert!(result.is_ok());

        // .rqbit dir should be gone
        assert!(!rqbit_dir.exists());
        // Download files should be gone
        assert!(!download_path.join("file1.dat").exists());
        assert!(!subdir.exists());
        // But the download directory itself should still exist
        assert!(download_path.exists());
    }

    #[tokio::test]
    async fn clear_storage_nonexistent_path_is_ok() {
        let mgr = TorrentManager::new();
        {
            let mut config = mgr.config.write();
            config.download_path = PathBuf::from("/tmp/nonexistent_test_path_xyz_12345");
        }
        let result = mgr.clear_storage().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn clear_storage_no_rqbit_dir_clears_files_only() {
        let tmp = TempDir::new().unwrap();
        let download_path = tmp.path().to_path_buf();

        let mgr = TorrentManager::new();
        {
            let mut config = mgr.config.write();
            config.download_path = download_path.clone();
        }

        // Only create download files, no .rqbit dir
        fs::write(download_path.join("file.dat"), "data").unwrap();

        let result = mgr.clear_storage().await;
        assert!(result.is_ok());
        assert!(!download_path.join("file.dat").exists());
        assert!(download_path.exists());
    }

    // ── Uninitialized error paths ───────────────────────────────────

    #[tokio::test]
    async fn add_fails_when_not_initialized() {
        let mgr = TorrentManager::new();
        let req = crate::types::AddTorrentRequest {
            source: "magnet:?xt=urn:btih:abc".to_string(),
            download_path: None,
            paused: None,
        };
        let result = mgr.add(req).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not initialized"));
    }

    #[tokio::test]
    async fn list_fails_when_not_initialized() {
        let mgr = TorrentManager::new();
        let result = mgr.list().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not initialized"));
    }

    #[tokio::test]
    async fn pause_fails_when_not_initialized() {
        let mgr = TorrentManager::new();
        let result = mgr.pause(0).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not initialized"));
    }

    #[tokio::test]
    async fn resume_fails_when_not_initialized() {
        let mgr = TorrentManager::new();
        let result = mgr.resume(0).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not initialized"));
    }

    #[tokio::test]
    async fn remove_fails_when_not_initialized() {
        let mgr = TorrentManager::new();
        let result = mgr.remove(0).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not initialized"));
    }

    #[tokio::test]
    async fn stats_fails_when_not_initialized() {
        let mgr = TorrentManager::new();
        let result = mgr.stats().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not initialized"));
    }

    #[tokio::test]
    async fn remove_all_returns_zero_when_not_initialized() {
        let mgr = TorrentManager::new();
        let result = mgr.remove_all().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn debug_info_when_not_initialized() {
        let mgr = TorrentManager::new();
        let result = mgr.debug_info().await;
        assert!(result.is_ok());
        let logs = result.unwrap();
        assert!(logs.len() >= 2);
        assert!(logs[0].contains("TORRENT DEBUG INFO"));
        assert!(logs[1].contains("NOT initialized"));
    }

    // ── Initialized session tests ───────────────────────────────────

    async fn create_initialized_manager() -> (TorrentManager, TempDir) {
        let tmp = TempDir::new().unwrap();
        let mgr = TorrentManager::new();
        let config = TorrentConfig {
            download_path: tmp.path().to_path_buf(),
            listen_port_range: 0..1, // port 0 lets the OS assign an available port
            enable_upnp: false,
            fast_resume: false,
            disable_dht_persistence: true,
            extra_trackers: vec![],
            http_api_bind_addr: None, // disable HTTP API for tests
        };
        mgr.initialize(config).await.unwrap();
        (mgr, tmp)
    }

    #[tokio::test]
    async fn initialize_sets_initialized() {
        let (mgr, _tmp) = create_initialized_manager().await;
        assert!(mgr.is_initialized());
    }

    #[tokio::test]
    async fn initialize_sets_download_path() {
        let (mgr, tmp) = create_initialized_manager().await;
        assert_eq!(mgr.download_path(), tmp.path().to_path_buf());
    }

    #[tokio::test]
    async fn initialize_creates_rqbit_dir() {
        let (_mgr, tmp) = create_initialized_manager().await;
        assert!(tmp.path().join(".rqbit").exists());
    }

    #[tokio::test]
    async fn list_returns_empty_on_fresh_session() {
        let (mgr, _tmp) = create_initialized_manager().await;
        let result = mgr.list().await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn stats_returns_zeros_on_fresh_session() {
        let (mgr, _tmp) = create_initialized_manager().await;
        let result = mgr.stats().await;
        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.total_downloaded, 0);
        assert_eq!(stats.total_uploaded, 0);
        assert_eq!(stats.download_speed, 0);
        assert_eq!(stats.upload_speed, 0);
        assert_eq!(stats.active_torrents, 0);
    }

    #[tokio::test]
    async fn remove_all_returns_zero_on_fresh_session() {
        let (mgr, _tmp) = create_initialized_manager().await;
        let result = mgr.remove_all().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn debug_info_when_initialized() {
        let (mgr, _tmp) = create_initialized_manager().await;
        let result = mgr.debug_info().await;
        assert!(result.is_ok());
        let logs = result.unwrap();
        assert!(logs.iter().any(|l| l.contains("Session is initialized")));
        assert!(logs.iter().any(|l| l.contains("API is available")));
        assert!(logs.iter().any(|l| l.contains("Torrent count: 0")));
        assert!(logs.iter().any(|l| l.contains("END DEBUG INFO")));
    }

    #[tokio::test]
    async fn pause_invalid_id_returns_error() {
        let (mgr, _tmp) = create_initialized_manager().await;
        let result = mgr.pause(99999).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn resume_invalid_id_returns_error() {
        let (mgr, _tmp) = create_initialized_manager().await;
        let result = mgr.resume(99999).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn remove_invalid_id_returns_error() {
        let (mgr, _tmp) = create_initialized_manager().await;
        let result = mgr.remove(99999).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn add_invalid_torrent_file_returns_error() {
        let (mgr, _tmp) = create_initialized_manager().await;
        let req = crate::types::AddTorrentRequest {
            source: "/nonexistent/file.torrent".to_string(),
            download_path: None,
            paused: None,
        };
        let result = mgr.add(req).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to load"));
    }

    #[tokio::test]
    async fn add_magnet_returns_initializing_info() {
        let (mgr, _tmp) = create_initialized_manager().await;
        let req = crate::types::AddTorrentRequest {
            source: "magnet:?xt=urn:btih:da39a3ee5e6b4b0d3255bfef95601890afd80709&dn=Test+File"
                .to_string(),
            download_path: None,
            paused: None,
        };
        let result = mgr.add(req).await;
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.name, "Test File");
        assert_eq!(
            info.info_hash,
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );
        assert_eq!(info.state, crate::types::TorrentState::Initializing);
        assert_eq!(info.progress, 0.0);
        assert!(info.output_path.is_some());
    }

    #[tokio::test]
    async fn add_magnet_sets_tracking_info() {
        let (mgr, _tmp) = create_initialized_manager().await;
        let req = crate::types::AddTorrentRequest {
            source: "magnet:?xt=urn:btih:da39a3ee5e6b4b0d3255bfef95601890afd80709&dn=Tracked"
                .to_string(),
            download_path: None,
            paused: None,
        };
        mgr.add(req).await.unwrap();
        let tracking =
            mgr.get_tracking_info("da39a3ee5e6b4b0d3255bfef95601890afd80709");
        assert!(tracking.is_some());
        assert!(tracking.unwrap().output_path.is_some());
    }

    #[tokio::test]
    async fn add_magnet_with_custom_download_path() {
        let (mgr, _tmp) = create_initialized_manager().await;
        let req = crate::types::AddTorrentRequest {
            source: "magnet:?xt=urn:btih:1111111111111111111111111111111111111111&dn=Custom"
                .to_string(),
            download_path: Some("/custom/path".to_string()),
            paused: None,
        };
        let info = mgr.add(req).await.unwrap();
        assert!(info.output_path.unwrap().starts_with("/custom/path"));
    }

    #[tokio::test]
    async fn add_invalid_magnet_no_btih_returns_error() {
        let (mgr, _tmp) = create_initialized_manager().await;
        let req = crate::types::AddTorrentRequest {
            source: "magnet:?xt=urn:sha1:abc123&dn=Bad".to_string(),
            download_path: None,
            paused: None,
        };
        let result = mgr.add(req).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Could not parse magnet"));
    }

    #[tokio::test]
    async fn initialize_with_extra_trackers() {
        let tmp = TempDir::new().unwrap();
        let mgr = TorrentManager::new();
        let config = TorrentConfig {
            download_path: tmp.path().to_path_buf(),
            listen_port_range: 0..1,
            enable_upnp: false,
            fast_resume: false,
            disable_dht_persistence: true,
            extra_trackers: vec![
                "udp://extra-tracker.example.com:1234/announce".to_string(),
            ],
            http_api_bind_addr: None,
        };
        let result = mgr.initialize(config).await;
        assert!(result.is_ok());
        assert!(mgr.is_initialized());
    }

    #[tokio::test]
    async fn initialize_with_invalid_extra_tracker_still_succeeds() {
        let tmp = TempDir::new().unwrap();
        let mgr = TorrentManager::new();
        let config = TorrentConfig {
            download_path: tmp.path().to_path_buf(),
            listen_port_range: 0..1,
            enable_upnp: false,
            fast_resume: false,
            disable_dht_persistence: true,
            extra_trackers: vec!["not a valid url".to_string()],
            http_api_bind_addr: None,
        };
        let result = mgr.initialize(config).await;
        assert!(result.is_ok());
    }

    // ── TrackingInfo ────────────────────────────────────────────────

    #[test]
    fn tracking_info_clone() {
        let info = TrackingInfo {
            output_path: Some("/path".to_string()),
        };
        let cloned = info.clone();
        assert_eq!(cloned.output_path, info.output_path);
    }

    #[test]
    fn tracking_info_debug() {
        let info = TrackingInfo {
            output_path: Some("/test".to_string()),
        };
        let debug = format!("{:?}", info);
        assert!(debug.contains("/test"));
    }

    #[test]
    fn tracking_info_none_output_path() {
        let info = TrackingInfo { output_path: None };
        assert!(info.output_path.is_none());
    }
}
