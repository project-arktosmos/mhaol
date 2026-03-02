use super::{Module, ModuleManifest, ModuleLinkSource, ModuleSettingDef};
use crate::AppState;
use mhaol_yt_dlp::{DownloadManager, YtDownloadConfig};
use std::sync::Arc;

pub struct YtdlModule {
    pub manager: Arc<DownloadManager>,
}

impl YtdlModule {
    pub fn new(config: YtDownloadConfig) -> Self {
        Self {
            manager: Arc::new(DownloadManager::new(config)),
        }
    }

    pub fn from_env() -> Self {
        Self::new(YtDownloadConfig::from_env())
    }
}

impl Module for YtdlModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            name: "yt-download".to_string(),
            version: "1.0.0".to_string(),
            description: "YouTube video/audio downloader".to_string(),
            source: Some("module".to_string()),
            compatibility: None,
            settings: vec![
                ModuleSettingDef {
                    key: "ytdl.downloadMode".to_string(),
                    default: "audio".to_string(),
                    env_key: None,
                },
                ModuleSettingDef {
                    key: "ytdl.quality".to_string(),
                    default: "best".to_string(),
                    env_key: None,
                },
                ModuleSettingDef {
                    key: "ytdl.format".to_string(),
                    default: "opus".to_string(),
                    env_key: None,
                },
                ModuleSettingDef {
                    key: "ytdl.videoQuality".to_string(),
                    default: "best".to_string(),
                    env_key: None,
                },
                ModuleSettingDef {
                    key: "ytdl.videoFormat".to_string(),
                    default: "mp4".to_string(),
                    env_key: None,
                },
                ModuleSettingDef {
                    key: "ytdl.poToken".to_string(),
                    default: String::new(),
                    env_key: Some("PO_TOKEN".to_string()),
                },
                ModuleSettingDef {
                    key: "ytdl.visitorData".to_string(),
                    default: String::new(),
                    env_key: Some("VISITOR_DATA".to_string()),
                },
                ModuleSettingDef {
                    key: "ytdl.cookies".to_string(),
                    default: String::new(),
                    env_key: Some("COOKIES".to_string()),
                },
            ],
            link_sources: vec![ModuleLinkSource {
                service: "youtube".to_string(),
                label: "YouTube".to_string(),
                media_type_id: "audio".to_string(),
                category_id: None,
            }],
            schema_sql: None,
        }
    }

    fn initialize(&self, _state: &AppState) -> Result<(), String> {
        tracing::info!("[ytdl] Download manager initialized");
        Ok(())
    }
}
