use super::{Module, ModuleManifest, ProcessStatus};
use crate::signaling_rooms::SignalingRoomManager;
use crate::AppState;
use std::sync::Arc;

pub struct SignalingModule {
    pub rooms: Arc<SignalingRoomManager>,
}

impl Module for SignalingModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            name: "signaling".to_string(),
            version: "0.0.1".to_string(),
            description: "Local WebRTC signaling server for peer-to-peer connections".to_string(),
            source: Some("module".to_string()),
            compatibility: None,
            settings: vec![],
            link_sources: vec![],
            schema_sql: None,
        }
    }

    fn processes(&self, _state: &AppState) -> Vec<ProcessStatus> {
        vec![ProcessStatus {
            id: "signaling".to_string(),
            available: self.rooms.is_available(),
            port: self.rooms.port(),
            url: self.rooms.dev_url(),
            log_prefix: "[signaling]".to_string(),
        }]
    }

    fn initialize(&self, _state: &AppState) -> Result<(), String> {
        Ok(())
    }
}
