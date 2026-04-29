use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::str::FromStr;

use futures::StreamExt;
use libp2p::core::muxing::StreamMuxerBox;
use libp2p::core::transport::{Boxed, OrTransport};
use libp2p::core::upgrade;
use libp2p::pnet::{PnetConfig, PreSharedKey};
use libp2p::{noise, tcp, yamux, Transport as Libp2pTransport};
use rust_ipfs::libp2p::Multiaddr;
use rust_ipfs::{Ipfs, IpfsPath, UninitializedIpfsDefault};

use crate::config::{IpfsConfig, DEFAULT_BOOTSTRAP};
use crate::types::{
    AddIpfsRequest, IpfsFileInfo, IpfsPeerInfo, IpfsState, IpfsStats,
};
use crate::util::{get_unix_timestamp, path_size_bytes, swarm_key_fingerprint};

/// Embedded IPFS node manager. Wraps a `rust_ipfs::Ipfs` handle so the
/// rest of the app can add files, pin/unpin CIDs, and query peers without
/// touching libp2p directly.
pub struct IpfsManager {
    config: RwLock<IpfsConfig>,
    state: RwLock<IpfsState>,
    /// The live IPFS handle. Held in a tokio RwLock because shutdown needs to
    /// take ownership across an await point (`exit_daemon` consumes self).
    ipfs: tokio::sync::RwLock<Option<Ipfs>>,
    /// Local cache of files added through `add_path`, keyed by root CID
    /// string. Survives only for the lifetime of the process.
    files: RwLock<HashMap<String, IpfsFileInfo>>,
    /// Cached identity captured at startup so health/status reads never
    /// hit the IPFS task channel.
    peer_id: RwLock<Option<String>>,
    listen_addrs: RwLock<Vec<String>>,
    /// Cached PSK fingerprint so the sync stats endpoint can return it
    /// without re-parsing the swarm key on every poll.
    swarm_key_fingerprint: RwLock<Option<String>>,
}

impl IpfsManager {
    pub fn new() -> Self {
        Self {
            config: RwLock::new(IpfsConfig::default()),
            state: RwLock::new(IpfsState::Stopped),
            ipfs: tokio::sync::RwLock::new(None),
            files: RwLock::new(HashMap::new()),
            peer_id: RwLock::new(None),
            listen_addrs: RwLock::new(Vec::new()),
            swarm_key_fingerprint: RwLock::new(None),
        }
    }

    pub fn is_private_network(&self) -> bool {
        self.config.read().is_private()
    }

    pub fn swarm_key_fingerprint(&self) -> Option<String> {
        self.swarm_key_fingerprint.read().clone()
    }

    pub fn state(&self) -> IpfsState {
        *self.state.read()
    }

    pub fn is_initialized(&self) -> bool {
        matches!(*self.state.read(), IpfsState::Running)
    }

    pub fn config(&self) -> IpfsConfig {
        self.config.read().clone()
    }

    pub fn peer_id(&self) -> Option<String> {
        self.peer_id.read().clone()
    }

    pub fn listen_addrs(&self) -> Vec<String> {
        self.listen_addrs.read().clone()
    }

    pub fn pinned_count(&self) -> u32 {
        self.files.read().values().filter(|f| f.pinned).count() as u32
    }

    pub fn repo_size_bytes(&self) -> u64 {
        let path = self.config.read().repo_path.clone();
        if path.as_os_str().is_empty() {
            return 0;
        }
        path_size_bytes(&path)
    }

    /// Build, configure, and start an IPFS node from `config`. Idempotent —
    /// calling twice with a live node first shuts the previous one down.
    ///
    /// When `config.swarm_key` is set the node joins a private swarm: the
    /// transport stack is replaced with TCP+pnet+noise+yamux, the public
    /// bootstrap list is skipped, and `bootstrap_on_start` is ignored.
    pub async fn initialize(&self, config: IpfsConfig) -> Result<()> {
        if self.is_initialized() {
            self.shutdown().await;
        }

        *self.state.write() = IpfsState::Starting;
        *self.config.write() = config.clone();

        // Parse and cache the swarm-key fingerprint up front so a malformed
        // key fails fast with a clear error instead of crashing inside the
        // libp2p task.
        let private_psk = match config.swarm_key.as_deref() {
            Some(key) => {
                let psk = PreSharedKey::from_str(key).map_err(|e| {
                    *self.state.write() = IpfsState::Error;
                    anyhow!("Invalid swarm key: {}", e)
                })?;
                let fingerprint = swarm_key_fingerprint(key).ok();
                *self.swarm_key_fingerprint.write() = fingerprint;
                Some(psk)
            }
            None => {
                *self.swarm_key_fingerprint.write() = None;
                None
            }
        };

        let mut builder = UninitializedIpfsDefault::new()
            .with_default()
            .set_default_listener();

        if config.enable_mdns {
            builder = builder.with_mdns();
        }

        if !config.repo_path.as_os_str().is_empty() {
            std::fs::create_dir_all(&config.repo_path).ok();
            builder = builder.set_path(&config.repo_path);
        }

        if config.listen_port != 0 {
            let listen: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", config.listen_port)
                .parse()
                .map_err(|e| anyhow!("Invalid listen multiaddr: {}", e))?;
            builder = builder.add_listening_addr(listen);
        }

        // Public-swarm bootstrap is meaningless on a private network — those
        // peers don't have our PSK and the connection would fail anyway.
        if private_psk.is_none() {
            let bootstrap_iter = DEFAULT_BOOTSTRAP
                .iter()
                .copied()
                .chain(config.extra_bootstrap.iter().map(|s| s.as_str()));
            for addr in bootstrap_iter {
                if let Ok(ma) = Multiaddr::from_str(addr) {
                    builder = builder.add_bootstrap(ma);
                } else {
                    log::warn!("[ipfs] skipping invalid bootstrap multiaddr: {}", addr);
                }
            }
        } else {
            // Private network: only honour user-supplied bootstrap entries.
            for addr in config.extra_bootstrap.iter() {
                if let Ok(ma) = Multiaddr::from_str(addr) {
                    builder = builder.add_bootstrap(ma);
                } else {
                    log::warn!("[ipfs] skipping invalid bootstrap multiaddr: {}", addr);
                }
            }
        }

        if let Some(psk) = private_psk {
            builder = builder.with_custom_transport(Box::new(move |keypair, relay| {
                build_pnet_transport(psk, keypair, relay)
            }));
        }

        let ipfs = builder.start().await.map_err(|e| {
            *self.state.write() = IpfsState::Error;
            anyhow!("Failed to start IPFS node: {}", e)
        })?;

        // Public bootstrap only makes sense for public swarms.
        if config.bootstrap_on_start && config.swarm_key.is_none() {
            if let Err(e) = ipfs.bootstrap().await {
                log::warn!("[ipfs] bootstrap failed: {}", e);
            }
        }

        // Cache identity + listen addrs so the sync health endpoint doesn't
        // need to await across the IPFS task channel on every status poll.
        match ipfs.identity(None).await {
            Ok(info) => {
                *self.peer_id.write() = Some(info.peer_id.to_string());
                *self.listen_addrs.write() =
                    info.listen_addrs.iter().map(|a| a.to_string()).collect();
            }
            Err(e) => log::warn!("[ipfs] identity lookup failed: {}", e),
        }

        *self.ipfs.write().await = Some(ipfs);
        *self.state.write() = IpfsState::Running;
        log::info!("[ipfs] node started");
        Ok(())
    }

    /// Stop the IPFS node and release the keypair. Safe to call when the
    /// node is not running.
    pub async fn shutdown(&self) {
        let handle = self.ipfs.write().await.take();
        if let Some(ipfs) = handle {
            ipfs.exit_daemon().await;
        }
        *self.state.write() = IpfsState::Stopped;
        *self.peer_id.write() = None;
        self.listen_addrs.write().clear();
        *self.swarm_key_fingerprint.write() = None;
    }

    fn handle(&self) -> Option<Ipfs> {
        // We hold a clone-able handle inside the lock so reads don't have to
        // be awaited under contention.
        self.ipfs.try_read().ok().and_then(|g| g.clone())
    }

    /// Add a file or directory at `request.source` to the IPFS blockstore.
    /// Returns the resulting CID. Pinning defaults to `true`.
    pub async fn add(&self, request: AddIpfsRequest) -> Result<IpfsFileInfo> {
        let ipfs = self
            .handle()
            .ok_or_else(|| anyhow!("IPFS node not initialized"))?;

        let path = std::path::PathBuf::from(&request.source);
        if !path.exists() {
            return Err(anyhow!("Source path does not exist: {}", request.source));
        }
        let pin = request.pin.unwrap_or(true);
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| request.source.clone());
        let size = path_size_bytes(&path);

        let ipfs_path: IpfsPath = ipfs
            .add_unixfs(path.as_path())
            .pin(pin)
            .await
            .map_err(|e| anyhow!("Failed to add to IPFS: {}", e))?;

        let cid_str = ipfs_path
            .root()
            .cid()
            .map(|c| c.to_string())
            .unwrap_or_else(|| ipfs_path.to_string());

        let info = IpfsFileInfo {
            cid: cid_str.clone(),
            name,
            size,
            pinned: pin,
            added_at: get_unix_timestamp(),
        };
        self.files.write().insert(cid_str, info.clone());
        Ok(info)
    }

    /// Pin a CID recursively. Idempotent — pinning an already-pinned CID
    /// returns Ok.
    pub async fn pin(&self, cid_str: &str) -> Result<()> {
        let ipfs = self
            .handle()
            .ok_or_else(|| anyhow!("IPFS node not initialized"))?;
        let cid = cid::Cid::from_str(cid_str)
            .map_err(|e| anyhow!("Invalid CID: {}", e))?;
        ipfs.insert_pin(&cid)
            .recursive()
            .await
            .map_err(|e| anyhow!("Failed to pin: {}", e))?;
        if let Some(info) = self.files.write().get_mut(cid_str) {
            info.pinned = true;
        }
        Ok(())
    }

    /// Remove a pin on a CID. Does not delete the underlying blocks; those
    /// are reaped by the next garbage collection.
    pub async fn unpin(&self, cid_str: &str) -> Result<()> {
        let ipfs = self
            .handle()
            .ok_or_else(|| anyhow!("IPFS node not initialized"))?;
        let cid = cid::Cid::from_str(cid_str)
            .map_err(|e| anyhow!("Invalid CID: {}", e))?;
        ipfs.remove_pin(&cid)
            .recursive()
            .await
            .map_err(|e| anyhow!("Failed to unpin: {}", e))?;
        if let Some(info) = self.files.write().get_mut(cid_str) {
            info.pinned = false;
        }
        Ok(())
    }

    /// List every pinned CID known to the node. Combines the IPFS blockstore
    /// view (authoritative) with whatever metadata we have cached locally.
    pub async fn list_pins(&self) -> Result<Vec<IpfsFileInfo>> {
        let ipfs = self
            .handle()
            .ok_or_else(|| anyhow!("IPFS node not initialized"))?;
        let mut stream = ipfs.list_pins(None).await;
        let cached = self.files.read().clone();

        let mut out: Vec<IpfsFileInfo> = Vec::new();
        while let Some(entry) = stream.next().await {
            if let Ok((cid, _mode)) = entry {
                let cid_str = cid.to_string();
                let info = cached.get(&cid_str).cloned().unwrap_or_else(|| IpfsFileInfo {
                    cid: cid_str.clone(),
                    name: cid_str,
                    size: 0,
                    pinned: true,
                    added_at: 0,
                });
                out.push(info);
            }
        }
        Ok(out)
    }

    /// Currently-connected peer count (libp2p connection-level peers).
    pub async fn connected_peer_count(&self) -> usize {
        let Some(ipfs) = self.handle() else {
            return 0;
        };
        ipfs.connected().await.map(|v| v.len()).unwrap_or(0)
    }

    /// Snapshot of currently-connected peers.
    pub async fn peers(&self) -> Result<Vec<IpfsPeerInfo>> {
        let ipfs = self
            .handle()
            .ok_or_else(|| anyhow!("IPFS node not initialized"))?;
        let addrs = ipfs
            .addrs()
            .await
            .map_err(|e| anyhow!("Failed to list peers: {}", e))?;
        Ok(addrs
            .into_iter()
            .map(|(peer, mut addrs)| IpfsPeerInfo {
                peer_id: peer.to_string(),
                addr: addrs.pop().map(|a| a.to_string()),
            })
            .collect())
    }

    /// Snapshot the manager's health/state for `/api/cloud/status`.
    pub async fn stats(&self) -> IpfsStats {
        let connected = self.connected_peer_count().await as u32;
        IpfsStats {
            state: self.state(),
            peer_id: self.peer_id(),
            agent_version: self.config.read().agent_version.clone(),
            connected_peers: connected,
            pinned_count: self.pinned_count(),
            repo_size_bytes: self.repo_size_bytes(),
            listen_addrs: self.listen_addrs(),
            private_network: self.is_private_network(),
            swarm_key_fingerprint: self.swarm_key_fingerprint(),
        }
    }
}

/// Build the libp2p transport pipeline for a private swarm: TCP -> pnet
/// handshake -> noise -> yamux. Returned as the `Boxed` transport rust-ipfs
/// expects from `with_custom_transport`. Relay is currently not supported on
/// private networks (the public relay nodes don't carry our PSK), so the
/// `relay` argument is intentionally ignored.
fn build_pnet_transport(
    psk: PreSharedKey,
    keypair: &libp2p::identity::Keypair,
    _relay: Option<libp2p::relay::client::Transport>,
) -> std::io::Result<Boxed<(libp2p::PeerId, StreamMuxerBox)>> {
    // Two independent TCP instances: one feeds the DNS resolver (for
    // `/dns4/...` addresses), the other handles raw `/ip4/.../tcp/...`
    // addresses directly. `tcp::tokio::Transport` is not `Clone`, so we
    // construct it twice instead.
    let tcp_for_dns = tcp::tokio::Transport::new(tcp::Config::new().nodelay(true));
    let tcp_raw = tcp::tokio::Transport::new(tcp::Config::new().nodelay(true));
    let dns = libp2p::dns::tokio::Transport::system(tcp_for_dns)
        .map_err(std::io::Error::other)?;
    let base = OrTransport::new(dns, tcp_raw);

    let pnet_cfg = PnetConfig::new(psk);
    let noise_cfg = noise::Config::new(keypair).map_err(std::io::Error::other)?;

    Ok(base
        .and_then(move |socket, _| pnet_cfg.handshake(socket))
        .upgrade(upgrade::Version::V1)
        .authenticate(noise_cfg)
        .multiplex(yamux::Config::default())
        .boxed())
}

impl Default for IpfsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_stopped_manager() {
        let mgr = IpfsManager::new();
        assert_eq!(mgr.state(), IpfsState::Stopped);
        assert!(!mgr.is_initialized());
        assert_eq!(mgr.pinned_count(), 0);
        assert!(mgr.peer_id().is_none());
        assert!(mgr.listen_addrs().is_empty());
    }

    #[test]
    fn default_creates_stopped_manager() {
        let mgr = IpfsManager::default();
        assert_eq!(mgr.state(), IpfsState::Stopped);
    }

    #[test]
    fn config_defaults() {
        let mgr = IpfsManager::new();
        let c = mgr.config();
        assert!(c.enable_mdns);
        assert!(c.bootstrap_on_start);
    }

    #[tokio::test]
    async fn add_fails_when_not_initialized() {
        let mgr = IpfsManager::new();
        let res = mgr
            .add(AddIpfsRequest {
                source: "/tmp/whatever".to_string(),
                pin: None,
            })
            .await;
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("not initialized"));
    }

    #[tokio::test]
    async fn pin_fails_when_not_initialized() {
        let mgr = IpfsManager::new();
        let res = mgr.pin("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn unpin_fails_when_not_initialized() {
        let mgr = IpfsManager::new();
        let res = mgr.unpin("bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi").await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn list_pins_fails_when_not_initialized() {
        let mgr = IpfsManager::new();
        assert!(mgr.list_pins().await.is_err());
    }

    #[tokio::test]
    async fn peers_fails_when_not_initialized() {
        let mgr = IpfsManager::new();
        assert!(mgr.peers().await.is_err());
    }

    #[tokio::test]
    async fn connected_peer_count_zero_when_uninitialized() {
        let mgr = IpfsManager::new();
        assert_eq!(mgr.connected_peer_count().await, 0);
    }

    #[tokio::test]
    async fn stats_reports_stopped_when_uninitialized() {
        let mgr = IpfsManager::new();
        let s = mgr.stats().await;
        assert_eq!(s.state, IpfsState::Stopped);
        assert_eq!(s.connected_peers, 0);
        assert_eq!(s.pinned_count, 0);
        assert!(!s.private_network);
        assert!(s.swarm_key_fingerprint.is_none());
    }

    #[tokio::test]
    async fn initialize_rejects_invalid_swarm_key() {
        let mgr = IpfsManager::new();
        let res = mgr
            .initialize(IpfsConfig {
                swarm_key: Some("not a key".to_string()),
                ..IpfsConfig::default()
            })
            .await;
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("swarm key"));
        assert_eq!(mgr.state(), IpfsState::Error);
        assert!(mgr.swarm_key_fingerprint().is_none());
    }

    #[tokio::test]
    async fn shutdown_when_uninitialized_is_noop() {
        let mgr = IpfsManager::new();
        mgr.shutdown().await;
        assert_eq!(mgr.state(), IpfsState::Stopped);
    }
}
