use crate::peer_service::rpc_handler::RpcHandler;
use crate::peer_service::rpc_types::RpcIncoming;
use crate::peer_service::types::DataChannelEnvelope;
use crate::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;

const AUTH_TIMESTAMP_MAX_AGE_MS: u64 = 30_000;

#[derive(Deserialize)]
pub struct AuthParams {
    address: String,
    signature: String,
    timestamp: String,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(ws_handler))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<AuthParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Validate timestamp
    let ts: u64 = match params.timestamp.parse() {
        Ok(t) => t,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid timestamp").into_response(),
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    if now.abs_diff(ts) > AUTH_TIMESTAMP_MAX_AGE_MS {
        return (StatusCode::UNAUTHORIZED, "Expired timestamp").into_response();
    }

    // Verify EIP-191 signature
    let message = format!("mhaol-rpc-auth:{}", params.timestamp);
    let recovered = match mhaol_identity::eip191_recover(&message, &params.signature) {
        Ok(addr) => addr,
        Err(_) => {
            return (StatusCode::UNAUTHORIZED, "Signature verification failed").into_response()
        }
    };

    if recovered.to_lowercase() != params.address.to_lowercase() {
        return (StatusCode::UNAUTHORIZED, "Signature mismatch").into_response();
    }

    let peer_address = recovered.to_lowercase();
    tracing::info!(address = %peer_address, "WS RPC connection authenticated");

    ws.on_upgrade(move |socket| handle_rpc_socket(socket, peer_address, state))
        .into_response()
}

async fn handle_rpc_socket(socket: WebSocket, peer_address: String, state: AppState) {
    let (mut ws_tx, mut ws_rx) = socket.split();
    let rpc_handler = RpcHandler::new(state);

    while let Some(result) = ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                tracing::debug!(error = %e, "WS RPC receive error");
                break;
            }
        };

        let text = match msg {
            Message::Text(t) => t,
            Message::Close(_) => break,
            Message::Ping(_) | Message::Pong(_) => continue,
            _ => continue,
        };

        let envelope: DataChannelEnvelope = match serde_json::from_str(&text) {
            Ok(e) => e,
            Err(e) => {
                tracing::debug!(error = %e, "Failed to parse WS envelope");
                continue;
            }
        };

        if envelope.channel != "rpc" {
            tracing::debug!(channel = %envelope.channel, "Ignoring non-rpc channel");
            continue;
        }

        let incoming: RpcIncoming = match serde_json::from_value(envelope.payload) {
            Ok(msg) => msg,
            Err(e) => {
                tracing::debug!(error = %e, "Failed to parse RPC payload");
                continue;
            }
        };

        let envelopes = rpc_handler
            .handle_message(incoming, Some(&peer_address))
            .await;

        for envelope in envelopes {
            let json = match serde_json::to_string(&envelope) {
                Ok(j) => j,
                Err(_) => continue,
            };
            if ws_tx.send(Message::Text(json.into())).await.is_err() {
                return;
            }
        }
    }

    tracing::info!(address = %peer_address, "WS RPC connection closed");
}
