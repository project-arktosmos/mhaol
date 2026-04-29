mod cloud_status;
mod database;
mod db;
mod documents;
mod frontend;
mod fs_browse;
mod health;
mod ipfs_pins;
mod libraries;
mod search;
mod state;

use axum::Router;
use mhaol_identity::IdentityManager;
use mhaol_queue::QueueManager;
use parking_lot::Mutex;
use rusqlite::Connection;
use state::CloudState;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;

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

    let db_path = std::env::var("DB_PATH")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("DATA_DIR")
                .ok()
                .map(|d| PathBuf::from(d).join("cloud-surrealkv"))
        })
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("mhaol")
                .join("cloud-surrealkv")
        });

    tracing::info!("Opening SurrealDB store at {}", db_path.display());
    let surreal = db::open(&db_path)
        .await
        .expect("Failed to initialize SurrealDB");

    let identities_dir = std::env::var("DATA_DIR")
        .ok()
        .map(|d| PathBuf::from(d).join("identities"))
        .unwrap_or_else(mhaol_identity::default_identities_dir);
    let signaling_url = std::env::var("SIGNALING_URL").unwrap_or_else(|_| {
        "https://mhaol-signaling.project-arktosmos.partykit.dev".to_string()
    });
    let identity_manager =
        IdentityManager::new(identities_dir, "cloud".to_string(), signaling_url);

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

    let queue = init_queue(&db_path);

    #[cfg(not(target_os = "android"))]
    let ytdl_manager = Arc::new(DownloadManager::new(YtDownloadConfig::from_env()));

    #[cfg(not(target_os = "android"))]
    let torrent_manager = {
        let manager = Arc::new(TorrentManager::new());
        let manager_clone = Arc::clone(&manager);
        let download_path = downloads_dir().join("torrents");
        tokio::spawn(async move {
            let config = TorrentConfig {
                download_path,
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
        let download_path = downloads_dir().join("ed2k");
        let config = Ed2kConfig {
            download_path,
            ..Ed2kConfig::default()
        };
        if let Err(e) = manager.initialize(config) {
            tracing::warn!("[ed2k] init failed: {}", e);
        }
        manager
    };

    #[cfg(not(target_os = "android"))]
    let ipfs_manager = {
        let manager = Arc::new(IpfsManager::new());
        let manager_clone = Arc::clone(&manager);
        let repo_path = downloads_dir().join("ipfs");
        let default_swarm_key_path = repo_path.join("swarm.key");
        tokio::spawn(async move {
            // The IPFS node always runs on a private network: read an
            // existing swarm key off disk or generate one on first boot. Copy
            // `<repo>/swarm.key` to any other node that should join the same
            // swarm. Override the path with `IPFS_SWARM_KEY_FILE`.
            std::fs::create_dir_all(&repo_path).ok();
            let key_path = std::env::var("IPFS_SWARM_KEY_FILE")
                .map(std::path::PathBuf::from)
                .unwrap_or(default_swarm_key_path);
            let swarm_key = match mhaol_ipfs::ensure_swarm_key(&key_path) {
                Ok(k) => Some(k),
                Err(e) => {
                    tracing::warn!(
                        "[ipfs] swarm key bootstrap failed at {}: {} — running on the public swarm",
                        key_path.display(),
                        e
                    );
                    None
                }
            };
            let config = IpfsConfig {
                repo_path,
                swarm_key,
                ..IpfsConfig::default()
            };
            if let Err(e) = manager_clone.initialize(config).await {
                tracing::warn!("[ipfs] init failed: {}", e);
            }
        });
        manager
    };

    let state = CloudState::new(
        surreal,
        identity_manager,
        queue,
        #[cfg(not(target_os = "android"))]
        ytdl_manager,
        #[cfg(not(target_os = "android"))]
        torrent_manager,
        #[cfg(not(target_os = "android"))]
        ed2k_manager,
        #[cfg(not(target_os = "android"))]
        ipfs_manager,
    );

    let app = Router::new()
        .nest("/api/health", health::router())
        .nest("/api/cloud", cloud_status::router())
        .nest("/api/libraries", libraries::router())
        .nest("/api/documents", documents::router())
        .nest("/api/database", database::router())
        .nest("/api/ipfs", ipfs_pins::router())
        .nest("/api/fs", fs_browse::router())
        .nest("/api/search", search::router())
        .fallback(frontend::serve_frontend)
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {}: {}", addr, e));

    tracing::info!("Cloud server (SurrealDB + web UI) listening on {}", addr);

    axum::serve(listener, app).await.expect("Server error");
}

fn init_queue(db_dir: &std::path::Path) -> Arc<QueueManager> {
    let queue_path = db_dir
        .parent()
        .map(|p| p.join("cloud-queue.db"))
        .unwrap_or_else(|| PathBuf::from("cloud-queue.db"));
    if let Some(parent) = queue_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let conn = Connection::open(&queue_path).expect("Failed to open queue SQLite");
    let pool = Arc::new(Mutex::new(conn));
    let manager = QueueManager::new(pool);
    manager.create_table();
    Arc::new(manager)
}

#[cfg(not(target_os = "android"))]
fn downloads_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("DATA_DIR") {
        let p = PathBuf::from(dir).join("downloads");
        std::fs::create_dir_all(&p).ok();
        return p;
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let p = manifest_dir.join("downloads");
    std::fs::create_dir_all(&p).ok();
    p
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
