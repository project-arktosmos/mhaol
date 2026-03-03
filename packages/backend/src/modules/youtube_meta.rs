use super::{Module, ModuleCompatibility, ModuleLinkSource, ModuleManifest};
use crate::db::schema::YOUTUBE_SCHEMA_SQL;

pub struct YoutubeMetaModule;

impl Module for YoutubeMetaModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            name: "youtube".to_string(),
            version: "1.0.0".to_string(),
            description: "YouTube video metadata via oEmbed".to_string(),
            source: Some("addon".to_string()),
            compatibility: Some(ModuleCompatibility {
                mobile: true,
                computer: true,
            }),
            settings: Vec::new(),
            link_sources: vec![ModuleLinkSource {
                service: "youtube".to_string(),
                label: "YouTube".to_string(),
                media_type_id: "audio".to_string(),
                category_id: None,
            }],
            schema_sql: Some(YOUTUBE_SCHEMA_SQL.to_string()),
        }
    }
}
