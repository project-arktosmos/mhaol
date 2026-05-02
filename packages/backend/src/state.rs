use mhaol_identity::IdentityManager;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

#[cfg(not(target_os = "android"))]
use crate::track_progress::AlbumProgressMap;
#[cfg(not(target_os = "android"))]
use mhaol_ipfs_core::IpfsManager;
#[cfg(not(target_os = "android"))]
use mhaol_ipfs_stream::manager::IpfsStreamManager;
#[cfg(not(target_os = "android"))]
use mhaol_torrent::TorrentManager;
#[cfg(not(target_os = "android"))]
use mhaol_yt_dlp::DownloadManager;
#[cfg(not(target_os = "android"))]
use std::sync::Arc;

/// Shared application state for the cloud server.
///
/// Backed by SurrealDB (embedded RocksDB).
#[derive(Clone)]
pub struct CloudState {
    pub db: Surreal<Db>,
    pub identity_manager: IdentityManager,
    #[cfg(not(target_os = "android"))]
    pub ytdl_manager: Arc<DownloadManager>,
    #[cfg(not(target_os = "android"))]
    pub torrent_manager: Arc<TorrentManager>,
    #[cfg(not(target_os = "android"))]
    pub ipfs_manager: Arc<IpfsManager>,
    #[cfg(not(target_os = "android"))]
    pub ipfs_stream_manager: Arc<IpfsStreamManager>,
    /// Live progress map for the per-firkin album track resolver. Keyed
    /// by the firkin id at the time the resolver was spawned (the
    /// bookmark id, pre-rollforward). The detail page polls this so
    /// each track's YouTube + lyrics status updates in real time as the
    /// background task resolves them.
    #[cfg(not(target_os = "android"))]
    pub track_progress: AlbumProgressMap,
}

impl CloudState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        db: Surreal<Db>,
        identity_manager: IdentityManager,
        #[cfg(not(target_os = "android"))] ytdl_manager: Arc<DownloadManager>,
        #[cfg(not(target_os = "android"))] torrent_manager: Arc<TorrentManager>,
        #[cfg(not(target_os = "android"))] ipfs_manager: Arc<IpfsManager>,
        #[cfg(not(target_os = "android"))] ipfs_stream_manager: Arc<IpfsStreamManager>,
    ) -> Self {
        Self {
            db,
            identity_manager,
            #[cfg(not(target_os = "android"))]
            ytdl_manager,
            #[cfg(not(target_os = "android"))]
            torrent_manager,
            #[cfg(not(target_os = "android"))]
            ipfs_manager,
            #[cfg(not(target_os = "android"))]
            ipfs_stream_manager,
            #[cfg(not(target_os = "android"))]
            track_progress: AlbumProgressMap::new(),
        }
    }
}
