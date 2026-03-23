use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use base64::Engine;
use hmac::{Hmac, Mac};
use serde::Serialize;
use sha1::Sha1;

use crate::config::Config;

#[derive(Debug, Clone, Serialize)]
pub struct IceServerEntry {
    pub urls: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
}

/// Generate time-limited TURN credentials using coturn's HMAC-SHA1 shared-secret mechanism.
pub fn generate_credentials(config: &Config) -> Vec<IceServerEntry> {
    if !config.has_turn() {
        return Vec::new();
    }

    let turn = &config.turn;

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

/// GET /api/v1/turn/credentials?apiKey=...
pub async fn turn_credentials_handler(
    State(config): State<std::sync::Arc<Config>>,
    axum::extract::Query(query): axum::extract::Query<CredentialQuery>,
) -> impl IntoResponse {
    // Validate API key if keys are configured
    if !config.auth.api_keys.is_empty() {
        match &query.api_key {
            Some(key) if config.auth.api_keys.contains(key) => {}
            _ => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({"error": "Invalid API Key"})),
                )
                    .into_response();
            }
        }
    }

    if !config.has_turn() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "TURN not configured"})),
        )
            .into_response();
    }

    let credentials = generate_credentials(&config);
    Json(credentials).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AuthConfig, ServerConfig, TurnConfig};

    fn test_config() -> Config {
        Config {
            server: ServerConfig::default(),
            turn: TurnConfig {
                domain: "turn.example.com".into(),
                shared_secret: "test-secret-key".into(),
                credential_ttl_secs: 86400,
                stun_port: 3478,
                turn_port: 3478,
                turns_port: 5349,
            },
            auth: AuthConfig {
                api_keys: vec!["test-api-key".into()],
            },
        }
    }

    #[test]
    fn generates_metered_compatible_format() {
        let config = test_config();
        let creds = generate_credentials(&config);

        assert_eq!(creds.len(), 2);

        // First entry: STUN (no credentials)
        assert!(creds[0].urls.as_str().unwrap().starts_with("stun:"));
        assert!(creds[0].username.is_none());
        assert!(creds[0].credential.is_none());

        // Second entry: TURN with credentials
        let urls = creds[1].urls.as_array().unwrap();
        assert_eq!(urls.len(), 3);
        assert!(urls[0].as_str().unwrap().starts_with("turn:"));
        assert!(urls[1].as_str().unwrap().contains("transport=tcp"));
        assert!(urls[2].as_str().unwrap().starts_with("turns:"));
        assert!(creds[1].username.is_some());
        assert!(creds[1].credential.is_some());

        // Username has expiry:mhaol format
        let username = creds[1].username.as_ref().unwrap();
        assert!(username.ends_with(":mhaol"));
        let expiry: u64 = username.split(':').next().unwrap().parse().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(expiry > now);
        assert!(expiry <= now + 86400 + 1);
    }

    #[test]
    fn returns_empty_when_turn_not_configured() {
        let config = Config {
            server: ServerConfig::default(),
            turn: TurnConfig::default(),
            auth: AuthConfig::default(),
        };
        let creds = generate_credentials(&config);
        assert!(creds.is_empty());
    }
}
