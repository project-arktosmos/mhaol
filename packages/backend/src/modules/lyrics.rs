use super::{Module, ModuleCompatibility, ModuleManifest};
use crate::db::schema::LYRICS_SCHEMA_SQL;

pub struct LyricsModule;

impl Module for LyricsModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            name: "lyrics".to_string(),
            version: "1.0.0".to_string(),
            description: "LrcLib lyrics fetching".to_string(),
            source: Some("addon".to_string()),
            compatibility: Some(ModuleCompatibility {
                mobile: true,
                computer: true,
            }),
            settings: Vec::new(),
            link_sources: Vec::new(),
            schema_sql: Some(LYRICS_SCHEMA_SQL.to_string()),
        }
    }
}
