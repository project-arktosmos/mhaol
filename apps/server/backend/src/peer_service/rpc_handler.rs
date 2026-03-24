use crate::AppState;
use axum::body::Body;
use axum::http::{Method, Request};
use axum::Router;
use tower::ServiceExt;

use super::rpc_types::{RpcIncoming, RpcOutgoing};
use super::types::DataChannelEnvelope;

const CHUNK_SIZE: usize = 15_000;

pub struct RpcHandler {
    router: Router,
}

impl RpcHandler {
    pub fn new(state: AppState) -> Self {
        let router = crate::api::build_router(state);
        Self { router }
    }

    pub async fn handle_message(
        &self,
        incoming: RpcIncoming,
    ) -> Vec<DataChannelEnvelope> {
        match incoming {
            RpcIncoming::Request {
                id,
                method,
                path,
                headers,
                body,
            } => self.handle_request(&id, &method, &path, headers.as_ref(), body.as_deref()).await,
            RpcIncoming::Subscribe { id, path } => {
                // SSE subscriptions handled separately via spawned task
                tracing::debug!(id = %id, path = %path, "RPC subscribe not yet implemented inline");
                vec![self.wrap(RpcOutgoing::StreamEnd { id })]
            }
            RpcIncoming::Unsubscribe { id } => {
                tracing::debug!(id = %id, "RPC unsubscribe");
                vec![]
            }
        }
    }

    async fn handle_request(
        &self,
        id: &str,
        method: &str,
        path: &str,
        headers: Option<&serde_json::Value>,
        body: Option<&str>,
    ) -> Vec<DataChannelEnvelope> {
        let method = method.parse::<Method>().unwrap_or(Method::GET);
        let mut req_builder = Request::builder().method(method).uri(path);

        if let Some(hdrs) = headers {
            if let Some(obj) = hdrs.as_object() {
                for (key, value) in obj {
                    if let Some(val_str) = value.as_str() {
                        req_builder = req_builder.header(key.as_str(), val_str);
                    }
                }
            }
        }

        let body = match body {
            Some(b) => Body::from(b.to_string()),
            None => Body::empty(),
        };

        let request = match req_builder.body(body) {
            Ok(r) => r,
            Err(e) => {
                return vec![self.wrap(RpcOutgoing::Response {
                    id: id.to_string(),
                    status: 400,
                    status_text: "Bad Request".to_string(),
                    headers: None,
                    body: Some(format!("{{\"error\":\"{}\"}}", e)),
                    chunked: None,
                    total_chunks: None,
                })];
            }
        };

        let response = match self.router.clone().oneshot(request).await {
            Ok(r) => r,
            Err(e) => {
                return vec![self.wrap(RpcOutgoing::Response {
                    id: id.to_string(),
                    status: 500,
                    status_text: "Internal Server Error".to_string(),
                    headers: None,
                    body: Some(format!("{{\"error\":\"{}\"}}", e)),
                    chunked: None,
                    total_chunks: None,
                })];
            }
        };

        let status = response.status();
        let status_text = status
            .canonical_reason()
            .unwrap_or("Unknown")
            .to_string();
        let status_code = status.as_u16();

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/json")
            .to_string();

        let mut resp_headers = serde_json::Map::new();
        resp_headers.insert(
            "content-type".to_string(),
            serde_json::Value::String(content_type.clone()),
        );

        let body_bytes = match axum::body::to_bytes(response.into_body(), 50 * 1024 * 1024).await {
            Ok(b) => b,
            Err(e) => {
                return vec![self.wrap(RpcOutgoing::Response {
                    id: id.to_string(),
                    status: 500,
                    status_text: "Internal Server Error".to_string(),
                    headers: None,
                    body: Some(format!("{{\"error\":\"Body read error: {}\"}}", e)),
                    chunked: None,
                    total_chunks: None,
                })];
            }
        };

        let body_str = String::from_utf8_lossy(&body_bytes).to_string();

        if body_str.len() > CHUNK_SIZE {
            return self.build_chunked_response(
                id,
                status_code,
                &status_text,
                &resp_headers,
                &body_str,
            );
        }

        vec![self.wrap(RpcOutgoing::Response {
            id: id.to_string(),
            status: status_code,
            status_text,
            headers: Some(serde_json::Value::Object(resp_headers)),
            body: Some(body_str),
            chunked: None,
            total_chunks: None,
        })]
    }

    fn build_chunked_response(
        &self,
        id: &str,
        status: u16,
        status_text: &str,
        headers: &serde_json::Map<String, serde_json::Value>,
        body: &str,
    ) -> Vec<DataChannelEnvelope> {
        let chunks: Vec<&str> = body
            .as_bytes()
            .chunks(CHUNK_SIZE)
            .map(|c| std::str::from_utf8(c).unwrap_or(""))
            .collect();
        let total = chunks.len();

        let mut envelopes = vec![self.wrap(RpcOutgoing::Response {
            id: id.to_string(),
            status,
            status_text: status_text.to_string(),
            headers: Some(serde_json::Value::Object(headers.clone())),
            body: None,
            chunked: Some(true),
            total_chunks: Some(total),
        })];

        for (seq, chunk) in chunks.iter().enumerate() {
            envelopes.push(self.wrap(RpcOutgoing::Chunk {
                id: id.to_string(),
                seq,
                data: chunk.to_string(),
                is_final: seq == total - 1,
            }));
        }

        envelopes
    }

    fn wrap(&self, msg: RpcOutgoing) -> DataChannelEnvelope {
        DataChannelEnvelope {
            channel: "rpc".to_string(),
            payload: serde_json::to_value(msg).unwrap_or_default(),
        }
    }
}
