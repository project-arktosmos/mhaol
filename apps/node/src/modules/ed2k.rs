use super::{Module, ModuleManifest};
use crate::AppState;
use mhaol_ed2k::Ed2kManager;
use std::sync::Arc;

pub struct Ed2kModule {
    pub manager: Arc<Ed2kManager>,
}

impl Default for Ed2kModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Ed2kModule {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(Ed2kManager::new()),
        }
    }
}

impl Module for Ed2kModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            name: "ed2k".to_string(),
            version: "0.1.0".to_string(),
            description: "eDonkey/eMule (ed2k) network client".to_string(),
            source: Some("module".to_string()),
            compatibility: None,
            settings: Vec::new(),
            link_sources: Vec::new(),
            schema_sql: None,
        }
    }

    fn initialize(&self, state: &AppState) -> Result<(), String> {
        let download_path = resolve_download_path(state);
        let cfg = mhaol_ed2k::Ed2kConfig {
            download_path: std::path::PathBuf::from(&download_path),
            ..Default::default()
        };
        if let Err(e) = self.manager.initialize(cfg) {
            tracing::error!("[ed2k] Failed to initialize: {}", e);
            return Err(e.to_string());
        }
        tracing::info!(
            "[ed2k] Engine initialized, download path: {}",
            download_path
        );
        Ok(())
    }

    fn shutdown(&self) {
        tracing::info!("[ed2k] Shutting down");
    }
}

fn resolve_download_path(state: &AppState) -> String {
    if let Some(lib_id) = state.metadata.get("ed2k.libraryId").map(|r| r.value) {
        if let Some(lib) = state.libraries.get(&lib_id) {
            return lib.path;
        }
    }
    if let Some(lib_id) = state.metadata.get("torrent.libraryId").map(|r| r.value) {
        if let Some(lib) = state.libraries.get(&lib_id) {
            return lib.path;
        }
    }
    let libs = state.libraries.get_all();
    if let Some(first) = libs.first() {
        return first.path.clone();
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    format!("{}/Downloads/mhaol", home)
}
