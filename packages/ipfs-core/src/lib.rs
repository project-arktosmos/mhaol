pub mod api;
pub mod config;
#[cfg(not(target_arch = "wasm32"))]
pub mod filestore;
pub mod manager;
pub mod types;
pub mod util;

pub use config::{IpfsConfig, DEFAULT_BOOTSTRAP};
#[cfg(not(target_arch = "wasm32"))]
pub use filestore::{
    compute_and_index_file_cid, FilestoreBlockStore, FilestoreEntry, FilestoreIndex,
    InMemoryFilestoreIndex,
};
pub use manager::IpfsManager;
pub use types::{
    AddIpfsRequest, IpfsFileInfo, IpfsPeerInfo, IpfsState, IpfsStats,
};
pub use util::{
    default_swarm_key_path, ensure_swarm_key, generate_swarm_key, load_swarm_key, save_swarm_key,
    swarm_key_fingerprint,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reexport_ipfs_config() {
        let _config: IpfsConfig = IpfsConfig::default();
    }

    #[test]
    fn reexport_default_bootstrap() {
        assert!(!DEFAULT_BOOTSTRAP.is_empty());
    }

    #[test]
    fn reexport_ipfs_manager() {
        let _mgr: IpfsManager = IpfsManager::new();
    }

    #[test]
    fn reexport_ipfs_state() {
        let _s = IpfsState::Stopped;
    }

    #[test]
    fn reexport_add_request() {
        let _r = AddIpfsRequest {
            source: "/tmp/file".to_string(),
            pin: Some(true),
        };
    }
}
