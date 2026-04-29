use mhaol_identity::IdentityManager;
use mhaol_queue::QueueManager;
use std::sync::Arc;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

#[cfg(not(target_os = "android"))]
use crate::worker_bridge::WorkerBridge;
#[cfg(not(target_os = "android"))]
use mhaol_ed2k::Ed2kManager;
#[cfg(not(target_os = "android"))]
use mhaol_ipfs::IpfsManager;
#[cfg(not(target_os = "android"))]
use mhaol_torrent::TorrentManager;
#[cfg(not(target_os = "android"))]
use mhaol_yt_dlp::DownloadManager;

/// Shared application state for the cloud server.
///
/// Backed by SurrealDB (embedded SurrealKV) — independent from the
/// SQLite-backed `mhaol-node` data layer.
#[derive(Clone)]
pub struct CloudState {
    pub db: Surreal<Db>,
    pub identity_manager: IdentityManager,
    pub queue: Arc<QueueManager>,
    #[cfg(not(target_os = "android"))]
    pub ytdl_manager: Arc<DownloadManager>,
    #[cfg(not(target_os = "android"))]
    pub torrent_manager: Arc<TorrentManager>,
    #[cfg(not(target_os = "android"))]
    pub ed2k_manager: Arc<Ed2kManager>,
    #[cfg(not(target_os = "android"))]
    pub ipfs_manager: Arc<IpfsManager>,
    #[cfg(not(target_os = "android"))]
    pub worker_bridge: Arc<WorkerBridge>,
    #[cfg(not(target_os = "android"))]
    pub signaling_url: String,
}

impl CloudState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        db: Surreal<Db>,
        identity_manager: IdentityManager,
        queue: Arc<QueueManager>,
        #[cfg(not(target_os = "android"))] ytdl_manager: Arc<DownloadManager>,
        #[cfg(not(target_os = "android"))] torrent_manager: Arc<TorrentManager>,
        #[cfg(not(target_os = "android"))] ed2k_manager: Arc<Ed2kManager>,
        #[cfg(not(target_os = "android"))] ipfs_manager: Arc<IpfsManager>,
        #[cfg(not(target_os = "android"))] worker_bridge: Arc<WorkerBridge>,
        #[cfg(not(target_os = "android"))] signaling_url: String,
    ) -> Self {
        Self {
            db,
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
            #[cfg(not(target_os = "android"))]
            worker_bridge,
            #[cfg(not(target_os = "android"))]
            signaling_url,
        }
    }
}
