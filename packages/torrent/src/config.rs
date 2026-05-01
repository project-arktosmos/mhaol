use std::ops::Range;
use std::path::PathBuf;

pub const DEFAULT_TRACKERS: &[&str] = &[
    "udp://tracker.opentrackr.org:1337/announce",
    "udp://open.stealth.si:80/announce",
    "udp://tracker.torrent.eu.org:451/announce",
    "udp://tracker.bittor.pw:1337/announce",
    "udp://public.popcorn-tracker.org:6969/announce",
    "udp://tracker.dler.org:6969/announce",
    "udp://exodus.desync.com:6969/announce",
    "udp://open.demonii.com:1337/announce",
];

#[derive(Debug, Clone)]
pub struct TorrentConfig {
    pub download_path: PathBuf,
    /// Separate directory used for "torrent stream" sessions so the temporary
    /// stream payloads stay isolated from real downloads and can be wiped on
    /// every fresh stream. If empty, a `streams/` subdirectory of
    /// `download_path` is used.
    pub stream_path: PathBuf,
    /// Port range for incoming peer connections (default: 6881..6891)
    pub listen_port_range: Range<u16>,
    /// Enable UPnP port forwarding (default: true)
    pub enable_upnp: bool,
    /// Enable fast resume (default: true)
    pub fast_resume: bool,
    /// Disable DHT persistence (default: true)
    pub disable_dht_persistence: bool,
    /// Additional tracker URLs beyond the defaults
    pub extra_trackers: Vec<String>,
}

impl Default for TorrentConfig {
    fn default() -> Self {
        Self {
            download_path: PathBuf::new(),
            stream_path: PathBuf::new(),
            listen_port_range: 6881..6891,
            enable_upnp: true,
            fast_resume: true,
            disable_dht_persistence: true,
            extra_trackers: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── DEFAULT_TRACKERS ────────────────────────────────────────────

    #[test]
    fn default_trackers_has_expected_count() {
        assert_eq!(DEFAULT_TRACKERS.len(), 8);
    }

    #[test]
    fn default_trackers_all_start_with_udp() {
        for tracker in DEFAULT_TRACKERS {
            assert!(
                tracker.starts_with("udp://"),
                "Tracker {} does not start with udp://",
                tracker
            );
        }
    }

    #[test]
    fn default_trackers_all_end_with_announce() {
        for tracker in DEFAULT_TRACKERS {
            assert!(
                tracker.ends_with("/announce"),
                "Tracker {} does not end with /announce",
                tracker
            );
        }
    }

    #[test]
    fn default_trackers_all_parseable_as_urls() {
        for tracker in DEFAULT_TRACKERS {
            let parsed: Result<url::Url, _> = tracker.parse();
            assert!(
                parsed.is_ok(),
                "Tracker {} failed to parse as URL: {:?}",
                tracker,
                parsed.err()
            );
        }
    }

    #[test]
    fn default_trackers_no_duplicates() {
        let mut seen = std::collections::HashSet::new();
        for tracker in DEFAULT_TRACKERS {
            assert!(
                seen.insert(tracker),
                "Duplicate tracker found: {}",
                tracker
            );
        }
    }

    // ── TorrentConfig::default() ────────────────────────────────────

    #[test]
    fn config_default_download_path_is_empty() {
        let config = TorrentConfig::default();
        assert_eq!(config.download_path, PathBuf::new());
    }

    #[test]
    fn config_default_listen_port_range() {
        let config = TorrentConfig::default();
        assert_eq!(config.listen_port_range, 6881..6891);
    }

    #[test]
    fn config_default_upnp_enabled() {
        let config = TorrentConfig::default();
        assert!(config.enable_upnp);
    }

    #[test]
    fn config_default_fast_resume_enabled() {
        let config = TorrentConfig::default();
        assert!(config.fast_resume);
    }

    #[test]
    fn config_default_dht_persistence_disabled() {
        let config = TorrentConfig::default();
        assert!(config.disable_dht_persistence);
    }

    #[test]
    fn config_default_no_extra_trackers() {
        let config = TorrentConfig::default();
        assert!(config.extra_trackers.is_empty());
    }

    // ── TorrentConfig custom values ─────────────────────────────────

    #[test]
    fn config_custom_values() {
        let config = TorrentConfig {
            download_path: PathBuf::from("/custom/path"),
            stream_path: PathBuf::from("/custom/streams"),
            listen_port_range: 7000..7010,
            enable_upnp: false,
            fast_resume: false,
            disable_dht_persistence: false,
            extra_trackers: vec!["udp://custom:1234/announce".to_string()],
        };
        assert_eq!(config.download_path, PathBuf::from("/custom/path"));
        assert_eq!(config.listen_port_range, 7000..7010);
        assert!(!config.enable_upnp);
        assert!(!config.fast_resume);
        assert!(!config.disable_dht_persistence);
        assert_eq!(config.extra_trackers.len(), 1);
    }

    #[test]
    fn config_clone() {
        let config = TorrentConfig {
            download_path: PathBuf::from("/test"),
            stream_path: PathBuf::from("/test-streams"),
            listen_port_range: 5000..5005,
            enable_upnp: false,
            fast_resume: true,
            disable_dht_persistence: true,
            extra_trackers: vec!["udp://extra:1234/announce".to_string()],
        };
        let cloned = config.clone();
        assert_eq!(cloned.download_path, config.download_path);
        assert_eq!(cloned.listen_port_range, config.listen_port_range);
        assert_eq!(cloned.enable_upnp, config.enable_upnp);
        assert_eq!(cloned.fast_resume, config.fast_resume);
        assert_eq!(cloned.disable_dht_persistence, config.disable_dht_persistence);
        assert_eq!(cloned.extra_trackers, config.extra_trackers);
    }

    #[test]
    fn config_debug() {
        let config = TorrentConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("TorrentConfig"));
        assert!(debug.contains("listen_port_range"));
    }
}
