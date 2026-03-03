use super::{Module, ModuleManifest, ProcessStatus};
use crate::signaling_dev::SignalingDevServer;
use crate::AppState;
use std::sync::Arc;

pub struct SignalingModule {
    pub dev_server: Arc<SignalingDevServer>,
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
            id: "signaling-dev".to_string(),
            available: self.dev_server.is_available(),
            port: 1999,
            url: self.dev_server.dev_url(),
            log_prefix: "[signaling-dev]".to_string(),
        }]
    }

    fn initialize(&self, _state: &AppState) -> Result<(), String> {
        let dev_server = Arc::clone(&self.dev_server);
        tokio::runtime::Handle::current().spawn(async move {
            dev_server.start().await;
        });
        Ok(())
    }
}
