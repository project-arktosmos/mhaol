pub mod api;
pub mod config;
pub mod manager;
pub mod types;
pub mod util;

pub use config::{IpfsConfig, DEFAULT_BOOTSTRAP};
pub use manager::IpfsManager;
pub use types::{
    AddIpfsRequest, IpfsFileInfo, IpfsPeerInfo, IpfsState, IpfsStats,
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
