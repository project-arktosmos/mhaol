use mhaol_identity::IdentityManager;
use parking_lot::Mutex as ParkingLotMutex;
use std::collections::HashMap;
use std::sync::Arc;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use tokio::sync::Mutex as TokioMutex;

#[cfg(not(target_os = "android"))]
use crate::album_download::AlbumDownloadProgressMap;
#[cfg(not(target_os = "android"))]
use crate::track_progress::AlbumProgressMap;
#[cfg(not(target_os = "android"))]
use crate::tv_build_progress::TvBuildProgressMap;
#[cfg(not(target_os = "android"))]
use crate::ytdl_channel_cache::YoutubeChannelCache;
#[cfg(not(target_os = "android"))]
use mhaol_ipfs_core::IpfsManager;
#[cfg(not(target_os = "android"))]
use mhaol_ipfs_stream::manager::IpfsStreamManager;
#[cfg(not(target_os = "android"))]
use mhaol_torrent::TorrentManager;
#[cfg(not(target_os = "android"))]
use mhaol_yt_dlp::DownloadManager;

/// Per-firkin async serialisers. Every code path that does a
/// read-modify-write on a firkin (`PUT /api/firkins/:id`,
/// `POST /api/firkins/:id/{files,subtitle,enrich,resolve-tracks,roms,finalize}`,
/// the torrent-completion watcher, the album-download task, the TV-build
/// task) **must** acquire the entry for the firkin's id before reading the
/// current record.
///
/// Without this serialisation, two concurrent mutations would each load
/// the same starting `files` array, mutate their local copy, and write
/// back — the later writer silently clobbers the earlier one. This is
/// exactly how a freshly-attached `torrent magnet` entry got dropped by
/// the trailer resolver's `youtube preferred client` write, leaving the
/// torrent permanently unmatched by `torrent_completion::run`.
pub type FirkinLockMap = Arc<ParkingLotMutex<HashMap<String, Arc<TokioMutex<()>>>>>;

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
    /// Live progress map for the per-firkin album download task. Keyed by
    /// firkin id (stable UUID, not CID). Drives the catalog detail page's
    /// "Download album" button + per-track download status badges.
    #[cfg(not(target_os = "android"))]
    pub album_download_progress: AlbumDownloadProgressMap,
    /// Live progress map for the background TV-show firkin builder. Keyed
    /// by `<library_id>::<lowercase_show>::<year>`. The libraries page
    /// polls this so each show group's "Match TMDB & build firkin"
    /// button surfaces phase + current/total counters in real time, and
    /// re-hydrates the in-progress badge after a refresh.
    #[cfg(not(target_os = "android"))]
    pub tv_build_progress: TvBuildProgressMap,
    /// In-memory cache for the YouTube channel RSS surface used by the
    /// catalog detail pages. Holds two layers: video id → channel id
    /// (long TTL since channel ownership is stable) and channel id →
    /// parsed feed (short TTL so the public Atom endpoint isn't
    /// hammered). See [`crate::ytdl_channel_cache`] for details.
    #[cfg(not(target_os = "android"))]
    pub ytdl_channel_cache: YoutubeChannelCache,
    /// Per-firkin async serialiser map. See [`FirkinLockMap`].
    pub firkin_locks: FirkinLockMap,
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
            #[cfg(not(target_os = "android"))]
            album_download_progress: AlbumDownloadProgressMap::new(),
            #[cfg(not(target_os = "android"))]
            tv_build_progress: TvBuildProgressMap::new(),
            #[cfg(not(target_os = "android"))]
            ytdl_channel_cache: YoutubeChannelCache::new(),
            firkin_locks: Arc::new(ParkingLotMutex::new(HashMap::new())),
        }
    }

    /// Acquire (or create) the per-firkin async lock for `id`. Callers
    /// hold the returned guard across the entire read-modify-write so
    /// concurrent mutations on the same firkin serialise. The outer
    /// parking-lot lock is held only for the map lookup.
    pub fn firkin_lock(&self, id: &str) -> Arc<TokioMutex<()>> {
        let mut map = self.firkin_locks.lock();
        map.entry(id.to_string())
            .or_insert_with(|| Arc::new(TokioMutex::new(())))
            .clone()
    }
}
