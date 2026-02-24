pub mod config;
pub mod manager;
pub mod types;
pub mod util;

pub use config::{TorrentConfig, DEFAULT_TRACKERS};
pub use manager::TorrentManager;
pub use types::{AddTorrentRequest, TorrentInfo, TorrentState, TorrentStats};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reexport_torrent_config() {
        let _config: TorrentConfig = TorrentConfig::default();
    }

    #[test]
    fn reexport_default_trackers() {
        assert!(!DEFAULT_TRACKERS.is_empty());
    }

    #[test]
    fn reexport_torrent_manager() {
        let _mgr: TorrentManager = TorrentManager::new();
    }

    #[test]
    fn reexport_add_torrent_request() {
        let _req: AddTorrentRequest = AddTorrentRequest {
            source: "test".to_string(),
            download_path: None,
            paused: None,
        };
    }

    #[test]
    fn reexport_torrent_info() {
        let _info: TorrentInfo = TorrentInfo {
            id: 0,
            name: "test".to_string(),
            info_hash: "hash".to_string(),
            size: 0,
            progress: 0.0,
            download_speed: 0,
            upload_speed: 0,
            peers: 0,
            seeds: 0,
            state: TorrentState::Initializing,
            added_at: 0,
            eta: None,
            output_path: None,
        };
    }

    #[test]
    fn reexport_torrent_state() {
        let _state: TorrentState = TorrentState::Downloading;
    }

    #[test]
    fn reexport_torrent_stats() {
        let _stats: TorrentStats = TorrentStats {
            total_downloaded: 0,
            total_uploaded: 0,
            download_speed: 0,
            upload_speed: 0,
            active_torrents: 0,
        };
    }
}
