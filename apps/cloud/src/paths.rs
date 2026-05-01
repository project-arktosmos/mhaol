//! Centralized on-disk layout for the cloud app.
//!
//! Everything the cloud writes — SurrealDB store, identities, IPFS repo,
//! torrent / ed2k / yt-dlp downloads, swarm key, rendezvous bootstrap file —
//! lives under a single root directory. By default this is
//! `<home>/mhaol-cloud/` (resolved per-OS via `dirs::home_dir()`); set
//! `DATA_DIR` to override.
//!
//! Individual paths can still be overridden one-by-one with their dedicated
//! env vars (`DB_PATH`, `IPFS_SWARM_KEY_FILE`, `RENDEZVOUS_BOOTSTRAP_FILE`,
//! `YTDL_OUTPUT_DIR`).

use std::path::PathBuf;

/// Directory name used under the home dir when `DATA_DIR` is not set.
const APP_DIR_NAME: &str = "mhaol-cloud";

/// Root directory for everything the cloud persists.
///
/// Resolution order:
/// 1. `DATA_DIR` env var, if set.
/// 2. `<dirs::home_dir()>/mhaol-cloud`.
/// 3. `./mhaol-cloud` as a last-resort fallback.
pub fn data_root() -> PathBuf {
    if let Ok(dir) = std::env::var("DATA_DIR") {
        return PathBuf::from(dir);
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(APP_DIR_NAME);
    }
    PathBuf::from(APP_DIR_NAME)
}

/// SurrealDB (RocksDB) store location. Override with `DB_PATH`.
pub fn db_path() -> PathBuf {
    if let Ok(p) = std::env::var("DB_PATH") {
        return PathBuf::from(p);
    }
    data_root().join("db")
}

/// Identity keystore directory.
pub fn identities_dir() -> PathBuf {
    data_root().join("identities")
}

/// Root downloads directory. Per-source subdirs hang off this.
pub fn downloads_dir() -> PathBuf {
    let p = data_root().join("downloads");
    std::fs::create_dir_all(&p).ok();
    p
}

pub fn torrents_dir() -> PathBuf {
    downloads_dir().join("torrents")
}

/// Scratch directory for "torrent stream" sessions. These are ephemeral —
/// each new `/api/torrent/stream` request wipes prior contents — so they
/// live separately from `torrents_dir()` to keep real downloads safe.
pub fn torrent_streams_dir() -> PathBuf {
    downloads_dir().join("torrent-streams")
}

pub fn ed2k_dir() -> PathBuf {
    downloads_dir().join("ed2k")
}

pub fn ipfs_repo_dir() -> PathBuf {
    downloads_dir().join("ipfs")
}

pub fn ipfs_stream_dir() -> PathBuf {
    downloads_dir().join("ipfs-stream")
}

pub fn youtube_dir() -> PathBuf {
    downloads_dir().join("youtube")
}

/// IPFS pre-shared swarm key. Override with `IPFS_SWARM_KEY_FILE`.
pub fn swarm_key_path() -> PathBuf {
    if let Ok(p) = std::env::var("IPFS_SWARM_KEY_FILE") {
        return PathBuf::from(p);
    }
    data_root().join("swarm.key")
}

/// File written by the rendezvous app announcing its dialable multiaddr.
/// Override with `RENDEZVOUS_BOOTSTRAP_FILE`.
pub fn rendezvous_bootstrap_file() -> PathBuf {
    if let Ok(p) = std::env::var("RENDEZVOUS_BOOTSTRAP_FILE") {
        return PathBuf::from(p);
    }
    data_root().join("rendezvous").join("bootstrap.multiaddr")
}
