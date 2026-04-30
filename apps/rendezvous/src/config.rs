use std::path::PathBuf;

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
}

impl RendezvousConfig {
    /// Read the rendezvous configuration from environment variables, falling
    /// back to per-OS defaults rooted at `<DATA_DIR>/rendezvous` (or
    /// `<home>/mhaol/rendezvous` when `DATA_DIR` is unset).
    pub fn from_env() -> Self {
        let host = std::env::var("RENDEZVOUS_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        let http_port: u16 = std::env::var("RENDEZVOUS_HTTP_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(DEFAULT_HTTP_PORT);

        let ipfs_listen_port: u16 = std::env::var("RENDEZVOUS_LISTEN_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(DEFAULT_LISTEN_PORT);

        let base = base_dir();
        let repo_path = std::env::var("RENDEZVOUS_REPO_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| base.join("ipfs"));

        let swarm_key_path = std::env::var("IPFS_SWARM_KEY_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| mhaol_ipfs::default_swarm_key_path());

        let bootstrap_file = std::env::var("RENDEZVOUS_BOOTSTRAP_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| base.join("bootstrap.multiaddr"));

        Self {
            host,
            http_port,
            ipfs_listen_port,
            repo_path,
            swarm_key_path,
            bootstrap_file,
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
