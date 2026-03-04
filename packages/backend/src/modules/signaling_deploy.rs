use super::{Module, ModuleManifest, ProcessStatus};
use crate::AppState;

pub struct SignalingDeployModule;

impl Module for SignalingDeployModule {
    fn manifest(&self) -> ModuleManifest {
        ModuleManifest {
            name: "signaling-deploy".to_string(),
            version: "0.0.1".to_string(),
            description: "PartyKit cloud deployment for the signaling server".to_string(),
            source: Some("module".to_string()),
            compatibility: None,
            settings: vec![],
            link_sources: vec![],
            schema_sql: None,
        }
    }

    fn processes(&self, state: &AppState) -> Vec<ProcessStatus> {
        state
            .signaling_servers
            .get_enabled()
            .into_iter()
            .map(|s| ProcessStatus {
                id: format!("signaling-remote-{}", s.id),
                available: true,
                port: 443,
                url: s.url,
                log_prefix: format!("[signaling:{}]", s.name),
            })
            .collect()
    }
}
