use super::{Module, ModuleManifest, ModuleSettingDef, ProcessStatus};
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
            settings: vec![
                ModuleSettingDef {
                    key: "signaling.partyUrl".to_string(),
                    default: String::new(),
                    env_key: None,
                },
                ModuleSettingDef {
                    key: "signaling.deployName".to_string(),
                    default: String::new(),
                    env_key: None,
                },
            ],
            link_sources: vec![],
            schema_sql: None,
        }
    }

    fn processes(&self, state: &AppState) -> Vec<ProcessStatus> {
        let party_url = state
            .settings
            .get("signaling.partyUrl")
            .unwrap_or_default();
        if party_url.is_empty() {
            return vec![];
        }
        vec![ProcessStatus {
            id: "signaling-partykit".to_string(),
            available: true,
            port: 443,
            url: party_url,
            log_prefix: "[signaling-deploy]".to_string(),
        }]
    }
}
