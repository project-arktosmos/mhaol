use super::{Module, ModuleManifest, ModuleSettingDef};
use crate::AppState;
use mhaol_torrent::TorrentManager;
use std::sync::Arc;

pub struct TorrentModule {
    pub manager: Arc<TorrentManager>,
}

impl Default for TorrentModule {
    fn default() -> Self {
        Self::new()
    }
}

impl TorrentModule {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(TorrentManager::new()),
        }
    }
}

impl Module for TorrentModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            name: "torrent".to_string(),
            version: "1.0.0".to_string(),
            description: "BitTorrent download client".to_string(),
            source: Some("module".to_string()),
            compatibility: None,
            settings: vec![ModuleSettingDef {
                key: "torrent.maxConnections".to_string(),
                default: "200".to_string(),
                env_key: None,
            }],
            link_sources: Vec::new(),
            schema_sql: None,
        }
    }

    fn initialize(&self, state: &AppState) -> Result<(), String> {
        // Resolve download path from metadata/library
        let download_path = resolve_download_path(state);

        let config = mhaol_torrent::TorrentConfig {
            download_path: std::path::PathBuf::from(&download_path),
            ..Default::default()
        };

        let manager = Arc::clone(&self.manager);
        tokio::spawn(async move {
            if let Err(e) = manager.initialize(config).await {
                tracing::error!("[torrent] Failed to initialize: {}", e);
            } else {
                tracing::info!(
                    "[torrent] Engine initialized, download path: {}",
                    download_path
                );
            }
        });

        Ok(())
    }

    fn shutdown(&self) {
        tracing::info!("[torrent] Shutting down");
    }
}

fn resolve_download_path(state: &AppState) -> String {
    // Try to get library path from metadata
    if let Some(lib_id) = state.metadata.get("torrent.libraryId").map(|r| r.value) {
        if let Some(lib) = state.libraries.get(&lib_id) {
            return lib.path;
        }
    }
    // Fallback to first library
    let libs = state.libraries.get_all();
    if let Some(first) = libs.first() {
        return first.path.clone();
    }
    // Fallback to ~/Downloads
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    format!("{}/Downloads/mhaol", home)
}
