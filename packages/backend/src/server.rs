use mhaol_backend::{api, AppState};
use std::path::PathBuf;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mhaol_backend=info".into()),
        )
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(1530);

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let db_path = std::env::var("DB_PATH").ok().map(PathBuf::from);

    let state = AppState::new(db_path.as_deref())
        .expect("Failed to initialize database");

    state.seed_default_library();
    state.initialize_modules();

    // Start local signaling dev server and p2p-stream worker in the background
    let signaling_dev = state.signaling_dev.clone();
    let worker_bridge = state.worker_bridge.clone();
    tokio::spawn(async move {
        signaling_dev.start().await;
        worker_bridge.start().await;
    });

    let app = api::build_router(state);

    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {}: {}", addr, e));

    tracing::info!("Backend server listening on {}", addr);

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
