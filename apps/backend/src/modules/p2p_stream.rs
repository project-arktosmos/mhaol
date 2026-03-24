use super::{Module, ModuleManifest, ModuleSettingDef};
use crate::AppState;

pub struct P2pStreamModule {
    initialized: std::sync::atomic::AtomicBool,
}

impl Default for P2pStreamModule {
    fn default() -> Self {
        Self::new()
    }
}

impl P2pStreamModule {
    pub fn new() -> Self {
        Self {
            initialized: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn is_available(&self) -> bool {
        self.initialized.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl Module for P2pStreamModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            name: "p2p-stream".to_string(),
            version: "1.0.0".to_string(),
            description: "WebRTC peer-to-peer media streaming".to_string(),
            source: Some("module".to_string()),
            compatibility: None,
            settings: vec![
                ModuleSettingDef {
                    key: "p2p-stream.stunServer".to_string(),
                    default: "stun:stun.l.google.com:19302".to_string(),
                    env_key: None,
                },
                ModuleSettingDef {
                    key: "p2p-stream.videoCodec".to_string(),
                    default: "vp8".to_string(),
                    env_key: None,
                },
                ModuleSettingDef {
                    key: "p2p-stream.videoQuality".to_string(),
                    default: "720p".to_string(),
                    env_key: None,
                },
            ],
            link_sources: Vec::new(),
            schema_sql: None,
        }
    }

    fn initialize(&self, _state: &AppState) -> Result<(), String> {
        // Initialize GStreamer
        match mhaol_p2p_stream::init() {
            Ok(()) => {
                self.initialized
                    .store(true, std::sync::atomic::Ordering::Relaxed);
                tracing::info!("[p2p-stream] GStreamer initialized");
                Ok(())
            }
            Err(e) => {
                tracing::warn!("[p2p-stream] GStreamer not available: {}", e);
                Ok(()) // Non-fatal - streaming just won't be available
            }
        }
    }
}
