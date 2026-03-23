use axum::{routing::get, Json, Router};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::config::Config;
use crate::rooms::RoomManager;
use crate::turn;
use crate::ws::{self, WsState};

fn build_router(config: Arc<Config>) -> Router {
    let rooms = Arc::new(RoomManager::new());
    let ws_state = WsState {
        rooms,
        config: config.clone(),
    };

    Router::new()
        .route("/party/{room_id}", get(ws::ws_handler))
        .route("/party/{room_id}/status", get(ws::room_status))
        .with_state(ws_state)
        .route(
            "/api/v1/turn/credentials",
            get(turn::turn_credentials_handler),
        )
        .with_state(config.clone())
        .route("/api/health", get(health))
        .layer(CorsLayer::permissive())
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "mhaol-signaling",
    }))
}

pub async fn run(config: Config) -> Result<(), String> {
    let config = Arc::new(config);
    let addr = format!("{}:{}", config.server.host, config.server.port);

    let app = build_router(config.clone());

    match (&config.server.tls_cert, &config.server.tls_key) {
        (Some(cert), Some(key)) => {
            tracing::info!("Starting signaling server with TLS on {addr}");
            let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(cert, key)
                .await
                .map_err(|e| format!("TLS config error: {e}"))?;
            let listener = std::net::TcpListener::bind(&addr)
                .map_err(|e| format!("Bind error: {e}"))?;
            axum_server::from_tcp_rustls(listener, tls_config)
                .serve(app.into_make_service())
                .await
                .map_err(|e| format!("Server error: {e}"))
        }
        _ => {
            tracing::info!("Starting signaling server on {addr} (no TLS)");
            let listener = tokio::net::TcpListener::bind(&addr)
                .await
                .map_err(|e| format!("Bind error: {e}"))?;
            axum::serve(listener, app)
                .await
                .map_err(|e| format!("Server error: {e}"))
        }
    }
}
