use super::{Module, ModuleCompatibility, ModuleManifest, ModuleSettingDef};

pub const RETROACHIEVEMENTS_SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS ra_game_list_cache (
    console_id INTEGER PRIMARY KEY,
    data TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS ra_game_details_cache (
    game_id INTEGER PRIMARY KEY,
    data TEXT NOT NULL,
    fetched_at TEXT NOT NULL DEFAULT (datetime('now'))
);
";

pub struct RetroachievementsModule;

impl Module for RetroachievementsModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            name: "retroachievements".to_string(),
            version: "1.0.0".to_string(),
            description: "RetroAchievements game metadata".to_string(),
            source: Some("addon".to_string()),
            compatibility: Some(ModuleCompatibility {
                mobile: true,
                computer: true,
            }),
            settings: vec![
                ModuleSettingDef {
                    key: "ra.apiUser".to_string(),
                    default: String::new(),
                    env_key: Some("RA_API_USER".to_string()),
                },
                ModuleSettingDef {
                    key: "ra.apiKey".to_string(),
                    default: String::new(),
                    env_key: Some("RA_API_KEY".to_string()),
                },
            ],
            link_sources: Vec::new(),
            schema_sql: Some(RETROACHIEVEMENTS_SCHEMA_SQL.to_string()),
        }
    }
}
