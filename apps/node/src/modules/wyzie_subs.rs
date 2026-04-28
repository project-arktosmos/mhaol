use super::{Module, ModuleCompatibility, ModuleManifest};
use crate::db::schema::SUBTITLES_SCHEMA_SQL;

pub struct WyzieSubsModule;

impl Module for WyzieSubsModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            name: "wyzie-subs".to_string(),
            version: "1.0.0".to_string(),
            description: "Wyzie Subs free subtitles search (movies + TV)".to_string(),
            source: Some("addon".to_string()),
            compatibility: Some(ModuleCompatibility {
                mobile: true,
                computer: true,
            }),
            settings: Vec::new(),
            link_sources: Vec::new(),
            schema_sql: Some(SUBTITLES_SCHEMA_SQL.to_string()),
        }
    }
}
