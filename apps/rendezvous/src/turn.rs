use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use base64::Engine;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use std::sync::Arc;

use crate::state::RendezvousState;

/// TURN credential generation + Metered-compatible REST endpoint.
///
/// Carries the same wire format as the legacy `mhaol-signaling` server, so any
/// client that speaks `GET /api/v1/turn/credentials?apiKey=...` and expects a
/// JSON array of `{urls, username?, credential?}` continues to work.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnConfig {
    /// TURN/STUN domain (e.g. `turn.example.com`). Empty disables TURN.
    #[serde(default)]
    pub domain: String,
    /// Shared secret matching coturn's `static-auth-secret`. Empty disables TURN.
    #[serde(default)]
    pub shared_secret: String,
    /// TTL for generated credentials, in seconds.
    #[serde(default = "default_ttl")]
    pub credential_ttl_secs: u64,
    #[serde(default = "default_stun_port")]
    pub stun_port: u16,
    #[serde(default = "default_turn_port")]
    pub turn_port: u16,
    #[serde(default = "default_turns_port")]
    pub turns_port: u16,
    /// Optional list of API keys accepted by the REST endpoint. Empty list
    /// means the endpoint is open (rendezvous already runs on a private
    /// swarm, so this is fine for LAN deployments).
    #[serde(default)]
    pub api_keys: Vec<String>,
}

impl Default for TurnConfig {
    fn default() -> Self {
        // Hand-rolled so a `TurnConfig::default()` call (used when env vars
        // override an absent TOML file) gets the same ports as the serde
        // defaults. `#[derive(Default)]` would zero the ports.
        Self {
            domain: String::new(),
            shared_secret: String::new(),
            credential_ttl_secs: default_ttl(),
            stun_port: default_stun_port(),
            turn_port: default_turn_port(),
            turns_port: default_turns_port(),
            api_keys: Vec::new(),
        }
    }
}

fn default_ttl() -> u64 {
    86_400
}
fn default_stun_port() -> u16 {
    3478
}
fn default_turn_port() -> u16 {
    3478
}
fn default_turns_port() -> u16 {
    5349
}

impl TurnConfig {
    pub fn is_configured(&self) -> bool {
        !self.domain.is_empty() && !self.shared_secret.is_empty()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct IceServerEntry {
    pub urls: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
}

/// Generate time-limited TURN credentials using coturn's HMAC-SHA1 shared-secret mechanism.
pub fn generate_credentials(turn: &TurnConfig) -> Vec<IceServerEntry> {
    if !turn.is_configured() {
        return Vec::new();
    }

    let expiry = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + turn.credential_ttl_secs;
    let username = format!("{expiry}:mhaol");

    let mut mac =
        Hmac::<Sha1>::new_from_slice(turn.shared_secret.as_bytes()).expect("HMAC accepts any key");
    mac.update(username.as_bytes());
    let credential = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());

    let domain = &turn.domain;
    let stun_port = turn.stun_port;
    let turn_port = turn.turn_port;
    let turns_port = turn.turns_port;

    vec![
        IceServerEntry {
            urls: serde_json::json!(format!("stun:{domain}:{stun_port}")),
            username: None,
            credential: None,
        },
        IceServerEntry {
            urls: serde_json::json!([
                format!("turn:{domain}:{turn_port}"),
                format!("turn:{domain}:{turn_port}?transport=tcp"),
                format!("turns:{domain}:{turns_port}?transport=tcp"),
            ]),
            username: Some(username),
            credential: Some(credential),
        },
    ]
}

/// Query params for the TURN credential REST API.
#[derive(serde::Deserialize)]
pub struct CredentialQuery {
    #[serde(rename = "apiKey")]
    api_key: Option<String>,
}

/// `GET /api/v1/turn/credentials?apiKey=...` — Metered-compatible response.
pub async fn turn_credentials_handler(
    State(state): State<RendezvousState>,
    axum::extract::Query(query): axum::extract::Query<CredentialQuery>,
) -> impl IntoResponse {
    let turn: Arc<TurnConfig> = state.turn.clone();

    if !turn.api_keys.is_empty() {
        match &query.api_key {
            Some(key) if turn.api_keys.contains(key) => {}
            _ => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({"error": "Invalid API Key"})),
                )
                    .into_response();
            }
        }
    }

    if !turn.is_configured() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "TURN not configured"})),
        )
            .into_response();
    }

    let credentials = generate_credentials(&turn);
    Json(credentials).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> TurnConfig {
        TurnConfig {
            domain: "turn.example.com".into(),
            shared_secret: "test-secret-key".into(),
            credential_ttl_secs: 86400,
            stun_port: 3478,
            turn_port: 3478,
            turns_port: 5349,
            api_keys: vec!["test-api-key".into()],
        }
    }

    #[test]
    fn generates_metered_compatible_format() {
        let creds = generate_credentials(&test_config());

        assert_eq!(creds.len(), 2);
        assert!(creds[0].urls.as_str().unwrap().starts_with("stun:"));
        assert!(creds[0].username.is_none());

        let urls = creds[1].urls.as_array().unwrap();
        assert_eq!(urls.len(), 3);
        assert!(urls[0].as_str().unwrap().starts_with("turn:"));
        assert!(urls[1].as_str().unwrap().contains("transport=tcp"));
        assert!(urls[2].as_str().unwrap().starts_with("turns:"));

        let username = creds[1].username.as_ref().unwrap();
        assert!(username.ends_with(":mhaol"));
    }

    #[test]
    fn returns_empty_when_turn_not_configured() {
        let creds = generate_credentials(&TurnConfig::default());
        assert!(creds.is_empty());
    }
}
