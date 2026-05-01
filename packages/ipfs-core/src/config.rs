use std::path::PathBuf;

/// Bootstrap multiaddrs for the public IPFS DHT. Picked from the official
/// Kubo defaults — the same nodes Kubo and rust-ipfs use out of the box.
pub const DEFAULT_BOOTSTRAP: &[&str] = &[
    "/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
    "/dnsaddr/bootstrap.libp2p.io/p2p/QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
    "/dnsaddr/bootstrap.libp2p.io/p2p/QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
    "/dnsaddr/bootstrap.libp2p.io/p2p/QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
    "/ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ",
];

/// Runtime configuration for the embedded IPFS node.
#[derive(Debug, Clone)]
pub struct IpfsConfig {
    /// On-disk repo path for the IPFS blockstore + datastore.
    /// When empty, the node starts in-memory (volatile across restarts).
    pub repo_path: PathBuf,
    /// TCP port to listen on for libp2p (`0` lets the OS pick).
    pub listen_port: u16,
    /// WebSocket-over-TCP port to listen on for browser-dialable libp2p
    /// connections. `0` disables the WS listener — when non-zero the node
    /// also binds `/ip4/0.0.0.0/tcp/<ws_listen_port>/ws`. The transport
    /// stack still goes through `pnet`+`noise`+`yamux`, so browser peers
    /// must present the same swarm key. Used by the rendezvous app to
    /// expose itself to the browser-side player.
    pub ws_listen_port: u16,
    /// Whether to enable mDNS local-network peer discovery.
    pub enable_mdns: bool,
    /// Whether to enable the Kademlia DHT.
    pub enable_kad_dht: bool,
    /// Whether to bootstrap against the public swarm on startup.
    pub bootstrap_on_start: bool,
    /// Additional bootstrap multiaddrs beyond `DEFAULT_BOOTSTRAP`.
    pub extra_bootstrap: Vec<String>,
    /// Display name advertised over Identify.
    pub agent_version: String,
    /// Pre-shared swarm key (go-ipfs `swarm.key` file format). When set, the
    /// node joins a private network only reachable by peers using the same
    /// key, the public bootstrap list is ignored, and the transport stack is
    /// constrained to TCP+pnet+noise+yamux (no QUIC/WebSocket). When `None`,
    /// the node behaves as a public IPFS node.
    pub swarm_key: Option<String>,
    /// Force the Kademlia DHT into Server mode after the node starts. Default
    /// `false` — leaves the node in `Auto` mode. Bootstrap nodes (rendezvous)
    /// must run in Server mode so other peers can store and look up records
    /// against them; regular peers can leave this off.
    pub dht_server_mode: bool,
}

impl Default for IpfsConfig {
    fn default() -> Self {
        Self {
            repo_path: PathBuf::new(),
            listen_port: 0,
            ws_listen_port: 0,
            enable_mdns: true,
            enable_kad_dht: true,
            bootstrap_on_start: true,
            extra_bootstrap: vec![],
            agent_version: format!("mhaol-ipfs-core/{}", env!("CARGO_PKG_VERSION")),
            swarm_key: None,
            dht_server_mode: false,
        }
    }
}

impl IpfsConfig {
    /// Convenience: returns true when the node is configured for a private
    /// swarm (i.e. has a swarm key).
    pub fn is_private(&self) -> bool {
        self.swarm_key.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bootstrap_non_empty() {
        assert!(!DEFAULT_BOOTSTRAP.is_empty());
        for addr in DEFAULT_BOOTSTRAP {
            assert!(addr.starts_with('/'), "Bootstrap {} not a multiaddr", addr);
        }
    }

    #[test]
    fn config_default_values() {
        let c = IpfsConfig::default();
        assert_eq!(c.listen_port, 0);
        assert!(c.enable_mdns);
        assert!(c.enable_kad_dht);
        assert!(c.bootstrap_on_start);
        assert!(c.extra_bootstrap.is_empty());
        assert!(c.agent_version.starts_with("mhaol-ipfs-core/"));
        assert_eq!(c.repo_path, PathBuf::new());
        assert!(c.swarm_key.is_none());
        assert!(!c.is_private());
    }

    #[test]
    fn is_private_reflects_swarm_key() {
        let mut c = IpfsConfig::default();
        assert!(!c.is_private());
        c.swarm_key = Some("anything".to_string());
        assert!(c.is_private());
    }

    #[test]
    fn config_clone() {
        let c = IpfsConfig::default();
        let c2 = c.clone();
        assert_eq!(c.listen_port, c2.listen_port);
        assert_eq!(c.agent_version, c2.agent_version);
    }
}
