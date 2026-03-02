use super::{Module, ModuleCompatibility, ModuleLinkSource, ModuleManifest, ModuleSettingDef};

pub struct TmdbModule;

impl Module for TmdbModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            name: "tmdb".to_string(),
            version: "1.0.0".to_string(),
            description: "The Movie Database (TMDB) metadata".to_string(),
            source: Some("addon".to_string()),
            compatibility: Some(ModuleCompatibility {
                mobile: true,
                computer: true,
            }),
            settings: vec![ModuleSettingDef {
                key: "tmdb.apiKey".to_string(),
                default: String::new(),
                env_key: Some("TMDB_API_KEY".to_string()),
            }],
            link_sources: vec![
                ModuleLinkSource {
                    service: "tmdb".to_string(),
                    label: "TMDB".to_string(),
                    media_type_id: "video".to_string(),
                    category_id: Some("movie".to_string()),
                },
                ModuleLinkSource {
                    service: "tmdb".to_string(),
                    label: "TMDB".to_string(),
                    media_type_id: "video".to_string(),
                    category_id: Some("tv_show".to_string()),
                },
            ],
            schema_sql: None,
        }
    }
}
