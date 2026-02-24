mod handlers;

use axum::{
    routing::{delete, get, post},
    Router,
};
use handlers::AppState;
use std::{collections::HashMap, env, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,mhaol_p2p_stream=debug".into()),
        )
        .init();

    mhaol_p2p_stream::init().expect("Failed to initialize GStreamer");
    info!("GStreamer initialized");

    let port: u16 = env::var("P2P_STREAM_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);

    let allowed_origins = env::var("P2P_STREAM_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:1530".into());

    let origins: Vec<_> = allowed_origins
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let state = Arc::new(AppState {
        sessions: RwLock::new(HashMap::new()),
    });

    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/sessions", post(handlers::create_session))
        .route("/sessions/{session_id}/ws", get(handlers::ws_handler))
        .route("/sessions/{session_id}", delete(handlers::delete_session))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("p2p-stream-server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server error");
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    info!("Shutdown signal received");
}
