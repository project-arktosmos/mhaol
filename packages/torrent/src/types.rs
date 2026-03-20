use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorrentInfo {
    pub id: usize,
    pub name: String,
    pub info_hash: String,
    pub size: u64,
    pub progress: f64,
    pub download_speed: u64,
    pub upload_speed: u64,
    pub peers: u32,
    pub seeds: u32,
    pub state: TorrentState,
    pub added_at: i64,
    pub eta: Option<u64>,
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TorrentState {
    Initializing,
    Downloading,
    Seeding,
    Paused,
    Checking,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AddTorrentRequest {
    pub source: String,
    pub download_path: Option<String>,
    pub paused: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorrentStats {
    pub total_downloaded: u64,
    pub total_uploaded: u64,
    pub download_speed: u64,
    pub upload_speed: u64,
    pub active_torrents: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TorrentFile {
    pub id: usize,
    pub name: String,
    pub size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── TorrentState ────────────────────────────────────────────────

    #[test]
    fn torrent_state_serialize_all_variants() {
        assert_eq!(
            serde_json::to_string(&TorrentState::Initializing).unwrap(),
            "\"initializing\""
        );
        assert_eq!(
            serde_json::to_string(&TorrentState::Downloading).unwrap(),
            "\"downloading\""
        );
        assert_eq!(
            serde_json::to_string(&TorrentState::Seeding).unwrap(),
            "\"seeding\""
        );
        assert_eq!(
            serde_json::to_string(&TorrentState::Paused).unwrap(),
            "\"paused\""
        );
        assert_eq!(
            serde_json::to_string(&TorrentState::Checking).unwrap(),
            "\"checking\""
        );
        assert_eq!(
            serde_json::to_string(&TorrentState::Error).unwrap(),
            "\"error\""
        );
    }

    #[test]
    fn torrent_state_deserialize_all_variants() {
        assert_eq!(
            serde_json::from_str::<TorrentState>("\"initializing\"").unwrap(),
            TorrentState::Initializing
        );
        assert_eq!(
            serde_json::from_str::<TorrentState>("\"downloading\"").unwrap(),
            TorrentState::Downloading
        );
        assert_eq!(
            serde_json::from_str::<TorrentState>("\"seeding\"").unwrap(),
            TorrentState::Seeding
        );
        assert_eq!(
            serde_json::from_str::<TorrentState>("\"paused\"").unwrap(),
            TorrentState::Paused
        );
        assert_eq!(
            serde_json::from_str::<TorrentState>("\"checking\"").unwrap(),
            TorrentState::Checking
        );
        assert_eq!(
            serde_json::from_str::<TorrentState>("\"error\"").unwrap(),
            TorrentState::Error
        );
    }

    #[test]
    fn torrent_state_deserialize_invalid() {
        assert!(serde_json::from_str::<TorrentState>("\"unknown\"").is_err());
    }

    #[test]
    fn torrent_state_clone_and_eq() {
        let state = TorrentState::Downloading;
        let cloned = state.clone();
        assert_eq!(state, cloned);
        assert_ne!(TorrentState::Downloading, TorrentState::Paused);
    }

    #[test]
    fn torrent_state_debug() {
        let debug = format!("{:?}", TorrentState::Seeding);
        assert_eq!(debug, "Seeding");
    }

    // ── TorrentInfo ─────────────────────────────────────────────────

    fn sample_torrent_info() -> TorrentInfo {
        TorrentInfo {
            id: 42,
            name: "test-torrent".to_string(),
            info_hash: "abc123def456".to_string(),
            size: 1024000,
            progress: 0.75,
            download_speed: 50000,
            upload_speed: 10000,
            peers: 12,
            seeds: 5,
            state: TorrentState::Downloading,
            added_at: 1700000000,
            eta: Some(300),
            output_path: Some("/downloads/test-torrent".to_string()),
        }
    }

    #[test]
    fn torrent_info_serialize_roundtrip() {
        let info = sample_torrent_info();
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: TorrentInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info, deserialized);
    }

    #[test]
    fn torrent_info_serialize_with_none_fields() {
        let info = TorrentInfo {
            id: 0,
            name: "minimal".to_string(),
            info_hash: "000".to_string(),
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
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"eta\":null"));
        assert!(json.contains("\"output_path\":null"));
        let deserialized: TorrentInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info, deserialized);
    }

    #[test]
    fn torrent_info_clone() {
        let info = sample_torrent_info();
        let cloned = info.clone();
        assert_eq!(info, cloned);
    }

    #[test]
    fn torrent_info_debug() {
        let info = sample_torrent_info();
        let debug = format!("{:?}", info);
        assert!(debug.contains("test-torrent"));
        assert!(debug.contains("abc123def456"));
    }

    #[test]
    fn torrent_info_deserialize_from_json_object() {
        let json = r#"{
            "id": 1,
            "name": "my file",
            "info_hash": "deadbeef",
            "size": 999,
            "progress": 1.0,
            "download_speed": 0,
            "upload_speed": 0,
            "peers": 0,
            "seeds": 3,
            "state": "seeding",
            "added_at": 1700000000,
            "eta": null,
            "output_path": "/tmp/my file"
        }"#;
        let info: TorrentInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.id, 1);
        assert_eq!(info.name, "my file");
        assert_eq!(info.state, TorrentState::Seeding);
        assert_eq!(info.eta, None);
        assert_eq!(info.output_path, Some("/tmp/my file".to_string()));
    }

    // ── AddTorrentRequest ───────────────────────────────────────────

    #[test]
    fn add_torrent_request_serialize_camel_case() {
        let req = AddTorrentRequest {
            source: "magnet:?xt=urn:btih:abc".to_string(),
            download_path: Some("/tmp/dl".to_string()),
            paused: Some(true),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"downloadPath\""));
        assert!(!json.contains("\"download_path\""));
    }

    #[test]
    fn add_torrent_request_deserialize_camel_case() {
        let json = r#"{"source":"magnet:test","downloadPath":"/tmp","paused":false}"#;
        let req: AddTorrentRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.source, "magnet:test");
        assert_eq!(req.download_path, Some("/tmp".to_string()));
        assert_eq!(req.paused, Some(false));
    }

    #[test]
    fn add_torrent_request_optional_fields_none() {
        let json = r#"{"source":"magnet:test"}"#;
        let req: AddTorrentRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.source, "magnet:test");
        assert_eq!(req.download_path, None);
        assert_eq!(req.paused, None);
    }

    #[test]
    fn add_torrent_request_roundtrip() {
        let req = AddTorrentRequest {
            source: "http://example.com/file.torrent".to_string(),
            download_path: None,
            paused: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: AddTorrentRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req, deserialized);
    }

    #[test]
    fn add_torrent_request_clone_and_debug() {
        let req = AddTorrentRequest {
            source: "test".to_string(),
            download_path: None,
            paused: Some(true),
        };
        let cloned = req.clone();
        assert_eq!(req, cloned);
        let debug = format!("{:?}", req);
        assert!(debug.contains("test"));
    }

    // ── TorrentStats ────────────────────────────────────────────────

    #[test]
    fn torrent_stats_serialize_roundtrip() {
        let stats = TorrentStats {
            total_downloaded: 1_000_000,
            total_uploaded: 500_000,
            download_speed: 125_000,
            upload_speed: 62_500,
            active_torrents: 3,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: TorrentStats = serde_json::from_str(&json).unwrap();
        assert_eq!(stats, deserialized);
    }

    #[test]
    fn torrent_stats_zero_values() {
        let stats = TorrentStats {
            total_downloaded: 0,
            total_uploaded: 0,
            download_speed: 0,
            upload_speed: 0,
            active_torrents: 0,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: TorrentStats = serde_json::from_str(&json).unwrap();
        assert_eq!(stats, deserialized);
    }

    #[test]
    fn torrent_stats_large_values() {
        let stats = TorrentStats {
            total_downloaded: u64::MAX,
            total_uploaded: u64::MAX,
            download_speed: u64::MAX,
            upload_speed: u64::MAX,
            active_torrents: u32::MAX,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: TorrentStats = serde_json::from_str(&json).unwrap();
        assert_eq!(stats, deserialized);
    }

    #[test]
    fn torrent_stats_clone_and_debug() {
        let stats = TorrentStats {
            total_downloaded: 100,
            total_uploaded: 50,
            download_speed: 10,
            upload_speed: 5,
            active_torrents: 1,
        };
        let cloned = stats.clone();
        assert_eq!(stats, cloned);
        let debug = format!("{:?}", stats);
        assert!(debug.contains("100"));
    }
}
