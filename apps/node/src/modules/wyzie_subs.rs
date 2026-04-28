use super::{Module, ModuleCompatibility, ModuleManifest, ModuleSettingDef};
use crate::db::schema::SUBTITLES_SCHEMA_SQL;

pub struct WyzieSubsModule;

impl Module for WyzieSubsModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            name: "wyzie-subs".to_string(),
            version: "1.0.0".to_string(),
            description: "Wyzie Subs subtitles search (movies + TV). Free key at https://sub.wyzie.io/redeem".to_string(),
            source: Some("addon".to_string()),
            compatibility: Some(ModuleCompatibility {
                mobile: true,
                computer: true,
            }),
            settings: vec![ModuleSettingDef {
                key: "wyzie-subs.apiKey".to_string(),
                default: String::new(),
                env_key: Some("WYZIE_API_KEY".to_string()),
            }],
            link_sources: Vec::new(),
            schema_sql: Some(SUBTITLES_SCHEMA_SQL.to_string()),
        }
    }
}
