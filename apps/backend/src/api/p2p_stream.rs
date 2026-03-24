use crate::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub fn router() -> Router<AppState> {
    Router::new().route("/settings", get(get_settings).put(put_settings))
}

const P2P_STREAM_SETTINGS_KEYS: &[&str] = &[
    "p2p-stream.stunServer",
    "p2p-stream.turnServers",
    "p2p-stream.videoCodec",
    "p2p-stream.audioCodec",
    "p2p-stream.defaultStreamMode",
    "p2p-stream.videoQuality",
];

fn defaults() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("p2p-stream.stunServer", "stun:stun.l.google.com:19302");
    m.insert("p2p-stream.turnServers", "[]");
    m.insert("p2p-stream.videoCodec", "vp8");
    m.insert("p2p-stream.audioCodec", "opus");
    m.insert("p2p-stream.defaultStreamMode", "video");
    m.insert("p2p-stream.videoQuality", "native");
    m
}

#[derive(Serialize)]
struct P2pStreamSettings {
    #[serde(rename = "stunServer")]
    stun_server: String,
    #[serde(rename = "turnServers")]
    turn_servers: serde_json::Value,
    #[serde(rename = "videoCodec")]
    video_codec: String,
    #[serde(rename = "audioCodec")]
    audio_codec: String,
    #[serde(rename = "defaultStreamMode")]
    default_stream_mode: String,
    #[serde(rename = "videoQuality")]
    video_quality: String,
}

async fn get_settings(State(state): State<AppState>) -> impl IntoResponse {
    let rows = state.settings.get_by_prefix("p2p-stream.");
    let existing: HashMap<String, String> = rows.into_iter().map(|r| (r.key, r.value)).collect();
    let defs = defaults();

    // Seed missing keys
    let mut missing = HashMap::new();
    for key in P2P_STREAM_SETTINGS_KEYS {
        if !existing.contains_key(*key) {
            missing.insert(key.to_string(), defs[key].to_string());
        }
    }
    if !missing.is_empty() {
        state.settings.set_many(&missing);
    }

    let get_val = |key: &str| -> String {
        existing
            .get(key)
            .cloned()
            .unwrap_or_else(|| defs[key].to_string())
    };

    let turn_servers_raw = get_val("p2p-stream.turnServers");
    let turn_servers: serde_json::Value =
        serde_json::from_str(&turn_servers_raw).unwrap_or(serde_json::json!([]));

    Json(P2pStreamSettings {
        stun_server: get_val("p2p-stream.stunServer"),
        turn_servers,
        video_codec: get_val("p2p-stream.videoCodec"),
        audio_codec: get_val("p2p-stream.audioCodec"),
        default_stream_mode: get_val("p2p-stream.defaultStreamMode"),
        video_quality: get_val("p2p-stream.videoQuality"),
    })
}

#[derive(Deserialize)]
struct UpdateSettings {
    #[serde(rename = "stunServer")]
    stun_server: Option<String>,
    #[serde(rename = "turnServers")]
    turn_servers: Option<serde_json::Value>,
    #[serde(rename = "videoCodec")]
    video_codec: Option<String>,
    #[serde(rename = "audioCodec")]
    audio_codec: Option<String>,
    #[serde(rename = "defaultStreamMode")]
    default_stream_mode: Option<String>,
    #[serde(rename = "videoQuality")]
    video_quality: Option<String>,
}

async fn put_settings(
    State(state): State<AppState>,
    Json(body): Json<UpdateSettings>,
) -> impl IntoResponse {
    let mut entries = HashMap::new();

    if let Some(v) = body.stun_server {
        entries.insert("p2p-stream.stunServer".to_string(), v);
    }
    if let Some(v) = body.turn_servers {
        entries.insert("p2p-stream.turnServers".to_string(), v.to_string());
    }
    if let Some(v) = body.video_codec {
        entries.insert("p2p-stream.videoCodec".to_string(), v);
    }
    if let Some(v) = body.audio_codec {
        entries.insert("p2p-stream.audioCodec".to_string(), v);
    }
    if let Some(v) = body.default_stream_mode {
        entries.insert("p2p-stream.defaultStreamMode".to_string(), v);
    }
    if let Some(v) = body.video_quality {
        entries.insert("p2p-stream.videoQuality".to_string(), v);
    }

    if entries.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "No valid settings provided" })),
        );
    }

    state.settings.set_many(&entries);
    (StatusCode::OK, Json(serde_json::json!({ "ok": true })))
}
