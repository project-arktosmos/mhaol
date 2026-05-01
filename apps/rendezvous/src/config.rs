use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::turn::TurnConfig;

/// Default TCP port the rendezvous IPFS node listens on. Stable so the cloud
/// app can default-bootstrap against `127.0.0.1:14001` without configuration.
pub const DEFAULT_LISTEN_PORT: u16 = 14001;

/// Default HTTP port the signaling API binds to.
pub const DEFAULT_HTTP_PORT: u16 = 14080;

#[derive(Debug, Clone)]
pub struct RendezvousConfig {
    pub host: String,
    pub http_port: u16,
    pub ipfs_listen_port: u16,
    pub repo_path: PathBuf,
    pub swarm_key_path: PathBuf,
    pub bootstrap_file: PathBuf,
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,
    pub turn: TurnConfig,
}

impl RendezvousConfig {
    /// Read the rendezvous configuration. An optional TOML file provides the
    /// base values; environment variables override them. Defaults fall back
    /// to per-OS paths rooted at `<DATA_DIR>/rendezvous` (or
    /// `<home>/mhaol/rendezvous` when `DATA_DIR` is unset).
    pub fn from_env_with_file<P: AsRef<Path>>(toml_path: Option<P>) -> Self {
        let file = toml_path
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| toml::from_str::<TomlConfig>(&s).ok())
            .unwrap_or_default();

        // Dual-stack default. Binding on `::` accepts both IPv4 and IPv6
        // connections on Linux/macOS (where `IPV6_V6ONLY` defaults to false).
        // The previous `0.0.0.0` default was IPv4-only; macOS resolves
        // `localhost` to `::1` first, and Firefox WebSockets don't fall back
        // to IPv4, which presented as `NS_ERROR_WEBSOCKET_CONNECTION_REFUSED`
        // even though tokio-based clients connected fine.
        let host = std::env::var("RENDEZVOUS_HOST")
            .ok()
            .or(file.server.as_ref().and_then(|s| s.host.clone()))
            .unwrap_or_else(|| "::".to_string());

        let http_port: u16 = std::env::var("RENDEZVOUS_HTTP_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .or(file.server.as_ref().and_then(|s| s.http_port))
            .unwrap_or(DEFAULT_HTTP_PORT);

        let ipfs_listen_port: u16 = std::env::var("RENDEZVOUS_LISTEN_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .or(file.server.as_ref().and_then(|s| s.ipfs_listen_port))
            .unwrap_or(DEFAULT_LISTEN_PORT);

        let base = base_dir();
        let repo_path = std::env::var("RENDEZVOUS_REPO_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| base.join("ipfs"));

        let swarm_key_path = std::env::var("IPFS_SWARM_KEY_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| mhaol_ipfs_core::default_swarm_key_path());

        let bootstrap_file = std::env::var("RENDEZVOUS_BOOTSTRAP_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| base.join("bootstrap.multiaddr"));

        let tls_cert = std::env::var("TLS_CERT")
            .ok()
            .or_else(|| file.server.as_ref().and_then(|s| s.tls_cert.clone()));
        let tls_key = std::env::var("TLS_KEY")
            .ok()
            .or_else(|| file.server.as_ref().and_then(|s| s.tls_key.clone()));

        let mut turn = file.turn.unwrap_or_default();
        if let Ok(v) = std::env::var("TURN_DOMAIN") {
            turn.domain = v;
        }
        if let Ok(v) = std::env::var("TURN_SHARED_SECRET") {
            turn.shared_secret = v;
        }
        if let Ok(v) = std::env::var("TURN_API_KEY") {
            if !v.is_empty() {
                turn.api_keys = vec![v];
            }
        }

        Self {
            host,
            http_port,
            ipfs_listen_port,
            repo_path,
            swarm_key_path,
            bootstrap_file,
            tls_cert,
            tls_key,
            turn,
        }
    }
}

fn base_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("DATA_DIR") {
        return PathBuf::from(dir).join("rendezvous");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join("mhaol").join("rendezvous");
    }
    PathBuf::from("rendezvous")
}

#[derive(Debug, Default, Deserialize)]
struct TomlConfig {
    #[serde(default)]
    server: Option<TomlServer>,
    #[serde(default)]
    turn: Option<TurnConfig>,
}

#[derive(Debug, Default, Deserialize)]
struct TomlServer {
    host: Option<String>,
    http_port: Option<u16>,
    ipfs_listen_port: Option<u16>,
    tls_cert: Option<String>,
    tls_key: Option<String>,
}
