mod artists;
mod catalog;
mod catalog_cache;
mod cloud_status;
mod database;
mod db;
mod disk;
#[cfg(not(target_os = "android"))]
mod filestore_index;
mod firkins;
mod frontend;
mod fs_browse;
mod health;
mod image_cache;
mod ipfs_pins;
mod ipfs_stream;
mod libraries;
#[cfg(not(target_os = "android"))]
mod library_scan;
mod media_trackers;
#[cfg(not(target_os = "android"))]
mod p2p;
mod paths;
mod player;
mod recommendations;
#[cfg(not(target_os = "android"))]
mod rom_extract;
mod search;
mod state;
#[cfg(not(target_os = "android"))]
mod tmdb_match;
#[cfg(not(target_os = "android"))]
mod tv_build;
#[cfg(not(target_os = "android"))]
mod tv_build_progress;
#[cfg(not(target_os = "android"))]
mod tv_match;
mod torrent;
mod torrent_completion;
#[cfg(not(target_os = "android"))]
mod track_progress;
#[cfg(not(target_os = "android"))]
mod track_resolve;
mod users;
#[cfg(not(target_os = "android"))]
mod ytdl;
#[cfg(not(target_os = "android"))]
mod ytdl_channel_cache;

use axum::Router;
use mhaol_identity::IdentityManager;
use state::CloudState;
use std::path::PathBuf;
#[cfg(not(target_os = "android"))]
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[cfg(not(target_os = "android"))]
use mhaol_ipfs_core::{IpfsConfig, IpfsManager};
#[cfg(not(target_os = "android"))]
use mhaol_torrent::{TorrentConfig, TorrentManager};
#[cfg(not(target_os = "android"))]
use mhaol_yt_dlp::{DownloadManager, YtDownloadConfig};

pub async fn run() {
    load_env();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,mhaol_backend=debug,surrealdb=info".into()),
        )
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(9898);

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let db_path = paths::db_path();

    tracing::info!("Cloud data root: {}", paths::data_root().display());
    tracing::info!("Opening SurrealDB store at {}", db_path.display());
    let surreal = db::open(&db_path)
        .await
        .expect("Failed to initialize SurrealDB");

    let identities_dir = paths::identities_dir();
    let identity_manager = IdentityManager::new(identities_dir, "cloud".to_string());

    identity_manager.ensure_identity("CLIENT_WALLET");

    #[cfg(not(target_os = "android"))]
    let ytdl_manager = {
        let mut config = YtDownloadConfig::from_env();
        if std::env::var("YTDL_OUTPUT_DIR").is_err() {
            config.output_path = paths::youtube_dir().to_string_lossy().into_owned();
        }
        std::fs::create_dir_all(&config.output_path).ok();
        Arc::new(DownloadManager::new(config))
    };

    #[cfg(not(target_os = "android"))]
    let torrent_manager = {
        let manager = Arc::new(TorrentManager::new());
        let manager_clone = Arc::clone(&manager);
        let download_path = paths::torrents_dir();
        let stream_path = paths::torrent_streams_dir();
        std::fs::create_dir_all(&stream_path).ok();
        tokio::spawn(async move {
            let config = TorrentConfig {
                download_path,
                stream_path,
                ..TorrentConfig::default()
            };
            if let Err(e) = manager_clone.initialize(config).await {
                tracing::warn!("[torrent] init failed: {}", e);
            }
        });
        manager
    };

    // Load the filestore index up-front so the IPFS node can start with a
    // populated `FilestoreBlockStore` decorator. Library-scan-recorded
    // leaves were saved to SurrealDB on previous runs; without loading
    // them here, bitswap would 404 every leaf until a fresh re-scan.
    #[cfg(not(target_os = "android"))]
    let filestore_index_for_ipfs: Option<Arc<dyn mhaol_ipfs_core::FilestoreIndex>> =
        match filestore_index::SurrealFilestoreIndex::load(surreal.clone()).await {
            Ok(idx) => Some(Arc::new(idx) as Arc<dyn mhaol_ipfs_core::FilestoreIndex>),
            Err(e) => {
                tracing::warn!(
                    "[filestore] failed to load index: {e} — IPFS will run without filestore decorator"
                );
                None
            }
        };

    #[cfg(not(target_os = "android"))]
    let ipfs_manager = {
        let manager = Arc::new(IpfsManager::new());
        let manager_clone = Arc::clone(&manager);
        let repo_path = paths::ipfs_repo_dir();
        let filestore_index_for_ipfs = filestore_index_for_ipfs.clone();
        tokio::spawn(async move {
            // The IPFS node always runs on a private network: read an
            // existing swarm key off disk or generate one on first boot.
            // Default location is `<data_root>/swarm.key`; override with
            // `IPFS_SWARM_KEY_FILE`.
            std::fs::create_dir_all(&repo_path).ok();
            let key_path = paths::swarm_key_path();
            if let Some(parent) = key_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            let swarm_key = match mhaol_ipfs_core::ensure_swarm_key(&key_path) {
                Ok(k) => k,
                Err(e) => {
                    tracing::error!(
                        "[ipfs] swarm key bootstrap failed at {}: {} — refusing to start IPFS on the public swarm",
                        key_path.display(),
                        e
                    );
                    return;
                }
            };
            // Cloud nodes find each other via mDNS on the LAN and gate every
            // connection on the swarm-key pnet handshake. Browsers reach the
            // node by dialing the `/ws` listener directly (the cloud's HTTP
            // server hands them the multiaddr via /api/p2p/bootstrap). No
            // standalone bootstrap peer is required.
            let listen_port: u16 = std::env::var("MHAOL_IPFS_TCP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(9900);
            let ws_listen_port: u16 = std::env::var("MHAOL_IPFS_WS_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(9901);
            let config = IpfsConfig {
                repo_path,
                swarm_key: Some(swarm_key),
                listen_port,
                ws_listen_port,
                enable_mdns: true,
                bootstrap_on_start: false,
                extra_bootstrap: vec![],
                filestore_index: filestore_index_for_ipfs,
                ..IpfsConfig::default()
            };
            if let Err(e) = manager_clone.initialize(config).await {
                tracing::warn!("[ipfs] init failed: {}", e);
            }
        });
        manager
    };

    #[cfg(not(target_os = "android"))]
    let ipfs_stream_manager = {
        if let Err(e) = mhaol_ipfs_stream::init() {
            tracing::warn!("[ipfs-stream] gstreamer init failed: {}", e);
        }
        let base_dir = paths::ipfs_stream_dir();
        std::fs::create_dir_all(&base_dir).ok();
        Arc::new(mhaol_ipfs_stream::manager::IpfsStreamManager::new(base_dir))
    };

    let state = CloudState::new(
        surreal,
        identity_manager,
        #[cfg(not(target_os = "android"))]
        ytdl_manager,
        #[cfg(not(target_os = "android"))]
        torrent_manager,
        #[cfg(not(target_os = "android"))]
        ipfs_manager,
        #[cfg(not(target_os = "android"))]
        Arc::clone(&ipfs_stream_manager),
    );

    #[cfg(not(target_os = "android"))]
    {
        let watcher_state = state.clone();
        tokio::spawn(async move {
            torrent_completion::run(watcher_state).await;
        });
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    #[allow(unused_mut)]
    let mut app = Router::new()
        .nest("/api/health", health::router())
        .nest("/api/cloud", cloud_status::router())
        .nest("/api/users", users::router())
        .nest("/api/libraries", libraries::router())
        .nest("/api/firkins", firkins::router())
        .nest("/api/media-trackers", media_trackers::router())
        .nest("/api/recommendations", recommendations::router())
        .nest("/api/artists", artists::router())
        .nest("/api/database", database::router())
        .nest("/api/disk", disk::router())
        .nest("/api/ipfs", ipfs_pins::router())
        .nest("/api/fs", fs_browse::router())
        .nest("/api/image-cache", image_cache::router())
        .nest("/api/search", search::router())
        .nest("/api/catalog", catalog::router())
        .nest("/api/torrent", torrent::router())
        .nest("/api/ipfs-stream", ipfs_stream::router())
        .nest("/api/player", player::router());

    #[cfg(not(target_os = "android"))]
    {
        app = app.nest("/api/ytdl", ytdl::router());
        app = app.nest("/api/p2p", p2p::router());
        // The tv_build router is nested under /api/libraries because its
        // routes are scoped by library id. Kept as a separate module so
        // the orchestrator + progress map don't drag library_scan-sized
        // imports into libraries.rs.
        app = app.nest("/api/libraries", tv_build::router());
    }

    let app = app
        .fallback(frontend::serve_frontend)
        .with_state(state)
        .layer(cors);

    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {}: {}", addr, e));

    tracing::info!("Cloud server (SurrealDB + web UI) listening on {}", addr);

    axum::serve(listener, app).await.expect("Server error");
}

/// Load .env from the workspace root into process environment variables.
/// Only sets variables that are not already present in the environment.
fn load_env() {
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let env_path = loop {
        if dir.join("pnpm-workspace.yaml").exists() {
            break dir.join(".env");
        }
        if !dir.pop() {
            break PathBuf::from(".env");
        }
    };

    let content = match std::fs::read_to_string(&env_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(eq_idx) = trimmed.find('=') {
            let key = trimmed[..eq_idx].trim();
            let value = trimmed[eq_idx + 1..].trim();
            if !key.is_empty() && std::env::var(key).is_err() {
                std::env::set_var(key, value);
            }
        }
    }
}
