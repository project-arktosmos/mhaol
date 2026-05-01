use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Ed2kServer {
    pub name: &'static str,
    pub host: &'static str,
    pub port: u16,
}

/// Currently-reachable eDonkey servers, ordered by user count (most populated
/// first) so source acquisition tries the busiest server before falling back.
/// Servers come and go; this list reflects what was alive at update time.
pub const DEFAULT_SERVERS: &[Ed2kServer] = &[
    Ed2kServer {
        name: "eMule Security",
        host: "45.82.80.155",
        port: 5687,
    },
    Ed2kServer {
        name: "eMule Sunrise",
        host: "176.123.5.89",
        port: 4725,
    },
    Ed2kServer {
        name: "Mazinga Server",
        host: "37.15.61.236",
        port: 4232,
    },
    Ed2kServer {
        name: "!! Sharing-Devils No.4 !!",
        host: "91.208.162.87",
        port: 4232,
    },
    Ed2kServer {
        name: "GrupoTS Server",
        host: "145.239.2.134",
        port: 4661,
    },
    Ed2kServer {
        name: "!! Sharing-Devils No.2 !!",
        host: "85.121.5.137",
        port: 4232,
    },
];

#[derive(Debug, Clone)]
pub struct Ed2kConfig {
    pub download_path: PathBuf,
    /// Listen port advertised to peers/servers (default: 4662 — eMule TCP).
    pub listen_port: u16,
    /// Per-attempt connection timeout in seconds when talking to servers.
    pub connect_timeout_secs: u64,
    /// User-supplied servers in addition to `DEFAULT_SERVERS`.
    pub extra_servers: Vec<Ed2kServer>,
    /// Display name advertised on login (default: "mhaol").
    pub user_name: String,
}

impl Default for Ed2kConfig {
    fn default() -> Self {
        Self {
            download_path: PathBuf::new(),
            listen_port: 4662,
            connect_timeout_secs: 8,
            extra_servers: vec![],
            user_name: "mhaol".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_servers_non_empty() {
        assert!(!DEFAULT_SERVERS.is_empty());
        for s in DEFAULT_SERVERS {
            assert!(!s.host.is_empty());
            assert!(s.port > 0);
        }
    }

    #[test]
    fn config_default_values() {
        let c = Ed2kConfig::default();
        assert_eq!(c.listen_port, 4662);
        assert_eq!(c.connect_timeout_secs, 8);
        assert!(c.extra_servers.is_empty());
        assert_eq!(c.user_name, "mhaol");
        assert_eq!(c.download_path, PathBuf::new());
    }

    #[test]
    fn config_clone() {
        let c = Ed2kConfig::default();
        let c2 = c.clone();
        assert_eq!(c.listen_port, c2.listen_port);
        assert_eq!(c.user_name, c2.user_name);
    }
}
