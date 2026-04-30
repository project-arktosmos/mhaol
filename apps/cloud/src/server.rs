mod catalog;
mod cloud_status;
mod database;
mod db;
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
mod p2p_stream;
mod paths;
mod player;
mod search;
mod state;
mod torrent;
mod torrent_completion;
mod worker_bridge;
#[cfg(not(target_os = "android"))]
mod ytdl;

use axum::Router;
use mhaol_identity::IdentityManager;
use state::CloudState;
use std::path::PathBuf;
#[cfg(not(target_os = "android"))]
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[cfg(not(target_os = "android"))]
use mhaol_ed2k::{Ed2kConfig, Ed2kManager};
#[cfg(not(target_os = "android"))]
use mhaol_ipfs::{IpfsConfig, IpfsManager};
#[cfg(not(target_os = "android"))]
use mhaol_torrent::{TorrentConfig, TorrentManager};
#[cfg(not(target_os = "android"))]
use mhaol_yt_dlp::{DownloadManager, YtDownloadConfig};

#[tokio::main]
async fn main() {
    #[cfg(not(target_os = "android"))]
    if std::env::args().nth(1).as_deref() == Some("worker") {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info,mhaol_p2p_stream=debug".into()),
            )
            .init();
        mhaol_p2p_stream::worker::run().await;
        return;
    }

    load_env();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,mhaol_cloud=debug,surrealdb=info".into()),
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
    let signaling_url = std::env::var("SIGNALING_URL")
        .unwrap_or_else(|_| "http://localhost:14080".to_string());
    let identity_manager =
        IdentityManager::new(identities_dir, "cloud".to_string(), signaling_url.clone());

    identity_manager.ensure_identity("SIGNALING_WALLET");
    identity_manager.ensure_identity("CLIENT_WALLET");

    if identity_manager.get_profile("SIGNALING_WALLET").is_none() {
        identity_manager.set_profile(
            "SIGNALING_WALLET",
            &mhaol_identity::Profile {
                username: "Cloud".to_string(),
                profile_picture_url: None,
            },
        );
    }

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

    #[cfg(not(target_os = "android"))]
    let ed2k_manager = {
        let manager = Arc::new(Ed2kManager::new());
        manager.install_arc();
        let download_path = paths::ed2k_dir();
        let config = Ed2kConfig {
            download_path,
            ..Ed2kConfig::default()
        };
        if let Err(e) = manager.initialize(config) {
            tracing::warn!("[ed2k] init failed: {}", e);
        }
        let manager_clone = Arc::clone(&manager);
        tokio::spawn(async move {
            loop {
                match manager_clone.connect_any_server().await {
                    Ok(server) => {
                        tracing::info!(
                            "[ed2k] connected to server {} ({}:{}), users={} files={}",
                            server.name,
                            server.host,
                            server.port,
                            server.user_count,
                            server.file_count
                        );
                        break;
                    }
                    Err(e) => {
                        tracing::warn!(
                            "[ed2k] no servers reachable: {} — retrying in 30s",
                            e
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                    }
                }
            }
        });
        manager
    };

    #[cfg(not(target_os = "android"))]
    let ipfs_manager = {
        let manager = Arc::new(IpfsManager::new());
        let manager_clone = Arc::clone(&manager);
        let repo_path = paths::ipfs_repo_dir();
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
            let swarm_key = match mhaol_ipfs::ensure_swarm_key(&key_path) {
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
            // Discover bootstrap multiaddrs for the rendezvous node:
            // `RENDEZVOUS_BOOTSTRAP` (newline- or comma-separated) takes
            // precedence, then the bootstrap file written by the rendezvous
            // app, then a localhost fallback so single-machine setups work
            // out of the box. Non-PSK peers cannot complete the libp2p
            // handshake, so this list is the only path the cloud uses to
            // discover other private-swarm peers.
            let extra_bootstrap = resolve_rendezvous_bootstrap();
            let config = IpfsConfig {
                repo_path,
                swarm_key: Some(swarm_key),
                // mDNS would still be filtered by the pnet handshake but
                // adds no value when we have a known rendezvous bootstrap.
                enable_mdns: false,
                bootstrap_on_start: true,
                extra_bootstrap,
                ..IpfsConfig::default()
            };
            if let Err(e) = manager_clone.initialize(config).await {
                tracing::warn!("[ipfs] init failed: {}", e);
            }
        });
        manager
    };

    #[cfg(not(target_os = "android"))]
    let worker_bridge = Arc::new(worker_bridge::WorkerBridge::new());

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
        ed2k_manager,
        #[cfg(not(target_os = "android"))]
        ipfs_manager,
        #[cfg(not(target_os = "android"))]
        Arc::clone(&worker_bridge),
        #[cfg(not(target_os = "android"))]
        Arc::clone(&ipfs_stream_manager),
        #[cfg(not(target_os = "android"))]
        signaling_url.clone(),
    );

    #[cfg(not(target_os = "android"))]
    {
        let watcher_state = state.clone();
        tokio::spawn(async move {
            torrent_completion::run(watcher_state).await;
        });
    }

    #[cfg(not(target_os = "android"))]
    {
        let bridge = Arc::clone(&worker_bridge);
        tokio::spawn(async move {
            bridge.start().await;
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
        .nest("/api/libraries", libraries::router())
        .nest("/api/firkins", firkins::router())
        .nest("/api/database", database::router())
        .nest("/api/ipfs", ipfs_pins::router())
        .nest("/api/fs", fs_browse::router())
        .nest("/api/image-cache", image_cache::router())
        .nest("/api/search", search::router())
        .nest("/api/catalog", catalog::router())
        .nest("/api/torrent", torrent::router())
        .nest("/api/p2p-stream", p2p_stream::router())
        .nest("/api/ipfs-stream", ipfs_stream::router())
        .nest("/api/player", player::router());

    #[cfg(not(target_os = "android"))]
    {
        app = app.nest("/api/ytdl", ytdl::router());
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

/// Build the bootstrap multiaddr list for the rendezvous IPFS node.
///
/// Sources, in order of precedence:
/// 1. `RENDEZVOUS_BOOTSTRAP` env var (newline- or comma-separated multiaddrs).
/// 2. The bootstrap file written by the rendezvous binary, default
///    `<data_root>/rendezvous/bootstrap.multiaddr`. Override with
///    `RENDEZVOUS_BOOTSTRAP_FILE`.
/// 3. A localhost default (`/ip4/127.0.0.1/tcp/14001`) so a single-machine
///    setup works without configuration. The peer-id portion is stripped, so
///    libp2p will dial-and-discover the actual peer id at runtime.
#[cfg(not(target_os = "android"))]
fn resolve_rendezvous_bootstrap() -> Vec<String> {
    if let Ok(raw) = std::env::var("RENDEZVOUS_BOOTSTRAP") {
        let entries: Vec<String> = raw
            .split(|c: char| c == '\n' || c == ',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !entries.is_empty() {
            return entries;
        }
    }

    let bootstrap_file = paths::rendezvous_bootstrap_file();

    if let Ok(contents) = std::fs::read_to_string(&bootstrap_file) {
        let entries: Vec<String> = contents
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !entries.is_empty() {
            return entries;
        }
    }

    // Localhost fallback. libp2p will dial this address and learn the peer
    // id from the noise handshake. The pnet layer rejects any peer that
    // doesn't carry the same swarm key, so even a wrong host on this port
    // cannot enter the swarm.
    vec!["/ip4/127.0.0.1/tcp/14001".to_string()]
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
