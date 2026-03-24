use super::{Module, ModuleCompatibility, ModuleManifest};

pub struct TorrentSearchModule;

impl Module for TorrentSearchModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            name: "torrent-search-thepiratebay".to_string(),
            version: "1.0.0".to_string(),
            description: "Torrent search via The Pirate Bay API".to_string(),
            source: Some("addon".to_string()),
            compatibility: Some(ModuleCompatibility {
                mobile: true,
                computer: true,
            }),
            settings: Vec::new(),
            link_sources: Vec::new(),
            schema_sql: None,
        }
    }
}
