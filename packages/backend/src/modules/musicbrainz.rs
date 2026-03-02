use super::{Module, ModuleCompatibility, ModuleLinkSource, ModuleManifest};

pub struct MusicbrainzModule;

impl Module for MusicbrainzModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            name: "musicbrainz".to_string(),
            version: "1.0.0".to_string(),
            description: "MusicBrainz music metadata".to_string(),
            source: Some("addon".to_string()),
            compatibility: Some(ModuleCompatibility {
                mobile: true,
                computer: true,
            }),
            settings: Vec::new(),
            link_sources: vec![ModuleLinkSource {
                service: "musicbrainz".to_string(),
                label: "MusicBrainz".to_string(),
                media_type_id: "audio".to_string(),
                category_id: None,
            }],
            schema_sql: None,
        }
    }
}
