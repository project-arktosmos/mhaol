use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub turn: TurnConfig,
    #[serde(default)]
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TurnConfig {
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub shared_secret: String,
    #[serde(default = "default_ttl")]
    pub credential_ttl_secs: u64,
    #[serde(default = "default_stun_port")]
    pub stun_port: u16,
    #[serde(default = "default_turn_port")]
    pub turn_port: u16,
    #[serde(default = "default_turns_port")]
    pub turns_port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    #[serde(default)]
    pub api_keys: Vec<String>,
}

fn default_host() -> String {
    "0.0.0.0".into()
}
fn default_port() -> u16 {
    8443
}
fn default_ttl() -> u64 {
    86400
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

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            tls_cert: None,
            tls_key: None,
        }
    }
}

impl Default for TurnConfig {
    fn default() -> Self {
        Self {
            domain: String::new(),
            shared_secret: String::new(),
            credential_ttl_secs: default_ttl(),
            stun_port: default_stun_port(),
            turn_port: default_turn_port(),
            turns_port: default_turns_port(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            api_keys: Vec::new(),
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, String> {
        let mut config = if path.exists() {
            let content =
                std::fs::read_to_string(path).map_err(|e| format!("Cannot read config: {e}"))?;
            toml::from_str::<Config>(&content).map_err(|e| format!("Invalid config: {e}"))?
        } else {
            tracing::info!("No config file found at {}, using defaults", path.display());
            Config {
                server: ServerConfig::default(),
                turn: TurnConfig::default(),
                auth: AuthConfig::default(),
            }
        };

        // Env var overrides
        if let Ok(v) = std::env::var("SIGNALING_HOST") {
            config.server.host = v;
        }
        if let Ok(v) = std::env::var("SIGNALING_PORT") {
            if let Ok(p) = v.parse() {
                config.server.port = p;
            }
        }
        if let Ok(v) = std::env::var("TLS_CERT") {
            config.server.tls_cert = Some(v);
        }
        if let Ok(v) = std::env::var("TLS_KEY") {
            config.server.tls_key = Some(v);
        }
        if let Ok(v) = std::env::var("TURN_DOMAIN") {
            config.turn.domain = v;
        }
        if let Ok(v) = std::env::var("TURN_SHARED_SECRET") {
            config.turn.shared_secret = v;
        }
        if let Ok(v) = std::env::var("TURN_API_KEY") {
            if !v.is_empty() {
                config.auth.api_keys = vec![v];
            }
        }

        Ok(config)
    }

    pub fn has_turn(&self) -> bool {
        !self.turn.domain.is_empty() && !self.turn.shared_secret.is_empty()
    }
}
