pub mod api;
pub mod client;
pub mod config;
pub mod manager;
pub mod types;
pub mod util;

pub use client::OfferedFile;
pub use config::{Ed2kConfig, Ed2kServer, DEFAULT_SERVERS};
pub use manager::Ed2kManager;
pub use types::{
    AddEd2kRequest, Ed2kFileInfo, Ed2kSearchResult, Ed2kState, Ed2kStats,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reexport_ed2k_config() {
        let _config: Ed2kConfig = Ed2kConfig::default();
    }

    #[test]
    fn reexport_default_servers() {
        assert!(!DEFAULT_SERVERS.is_empty());
    }

    #[test]
    fn reexport_ed2k_manager() {
        let _mgr: Ed2kManager = Ed2kManager::new();
    }

    #[test]
    fn reexport_add_ed2k_request() {
        let _req = AddEd2kRequest {
            source: "ed2k://|file|test|0|aabbccdd|/".to_string(),
            download_path: None,
            paused: None,
        };
    }

    #[test]
    fn reexport_ed2k_state() {
        let _s = Ed2kState::Initializing;
    }
}
