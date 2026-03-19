use super::{Module, ModuleCompatibility, ModuleManifest, ModuleSettingDef};

pub struct JackettModule;

impl Module for JackettModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            name: "jackett".to_string(),
            version: "1.0.0".to_string(),
            description: "Torrent search via Jackett indexer proxy".to_string(),
            source: Some("addon".to_string()),
            compatibility: Some(ModuleCompatibility {
                mobile: true,
                computer: true,
            }),
            settings: vec![
                ModuleSettingDef {
                    key: "jackett.apiUrl".to_string(),
                    default: "http://localhost:9117".to_string(),
                    env_key: Some("JACKETT_API_URL".to_string()),
                },
                ModuleSettingDef {
                    key: "jackett.apiKey".to_string(),
                    default: String::new(),
                    env_key: Some("JACKETT_API_KEY".to_string()),
                },
            ],
            link_sources: Vec::new(),
            schema_sql: None,
        }
    }
}
