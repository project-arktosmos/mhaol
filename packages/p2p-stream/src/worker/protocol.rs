use serde::{Deserialize, Serialize};

/// Commands received from SvelteKit via stdin (one JSON object per line).
#[derive(Debug, Deserialize)]
#[serde(tag = "command")]
pub enum Command {
    #[serde(rename = "create_session")]
    CreateSession {
        session_id: String,
        file_path: String,
        mode: Option<String>,
        video_codec: Option<String>,
        video_quality: Option<String>,
        signaling_url: String,
    },
    #[serde(rename = "delete_session")]
    DeleteSession { session_id: String },
}

/// Events sent to SvelteKit via stdout (one JSON object per line).
#[derive(Debug, Serialize)]
#[serde(tag = "event")]
pub enum Event {
    #[serde(rename = "session_created")]
    SessionCreated { session_id: String, room_id: String },
    #[serde(rename = "session_deleted")]
    SessionDeleted { session_id: String },
    #[serde(rename = "error")]
    Error {
        session_id: Option<String>,
        error: String,
    },
}

impl Event {
    pub fn to_json_line(&self) -> String {
        let mut json = serde_json::to_string(self).expect("Event must serialize");
        json.push('\n');
        json
    }
}
