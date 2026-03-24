use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum RpcIncoming {
    Request {
        id: String,
        method: String,
        path: String,
        #[serde(default)]
        headers: Option<serde_json::Value>,
        #[serde(default)]
        body: Option<String>,
    },
    Subscribe {
        id: String,
        path: String,
    },
    Unsubscribe {
        id: String,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum RpcOutgoing {
    Response {
        id: String,
        status: u16,
        #[serde(rename = "statusText")]
        status_text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        body: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        chunked: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none", rename = "totalChunks")]
        total_chunks: Option<usize>,
    },
    Chunk {
        id: String,
        seq: usize,
        data: String,
        #[serde(rename = "final")]
        is_final: bool,
    },
    StreamEvent {
        id: String,
        #[serde(rename = "eventType")]
        event_type: String,
        data: String,
    },
    StreamEnd {
        id: String,
    },
}
