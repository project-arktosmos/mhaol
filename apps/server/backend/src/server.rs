use mhaol_server::{api, load_env_app, AppState};
use std::path::PathBuf;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // When invoked as `mhaol-server worker`, run the p2p-stream worker loop
    // instead of the HTTP server. This allows shipping a single binary.
    #[cfg(not(target_os = "android"))]
    if std::env::args().nth(1).as_deref() == Some("worker") {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info,mhaol_p2p_stream=debug".into()),
            )
            .init();
        mhaol_p2p_stream::worker::run().await;
        return;
    }

    load_env_app();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,mhaol_server=debug,mhaol_torrent=debug,mhaol_http_stream=debug,mhaol_p2p_stream=debug,librqbit=info".into()),
        )
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(1530);

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let db_path = std::env::var("DB_PATH")
        .ok()
        .map(PathBuf::from)
        .or_else(|| std::env::var("DATA_DIR").ok().map(|d| PathBuf::from(d).join("mhaol.db")))
        .unwrap_or_else(|| {
            // Default: packages/database/mhaol.db relative to the workspace root.
            // The server binary lives at apps/server/backend/, so go up three levels.
            // If the workspace path doesn't exist (e.g. distributed binary), use ./mhaol.db.
            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            let candidate = PathBuf::from(manifest_dir)
                .parent() // apps/server/
                .and_then(|p| p.parent()) // apps/
                .and_then(|p| p.parent()) // workspace root
                .map(|root| root.join("packages/database/mhaol.db"));
            match candidate {
                Some(p) if p.parent().is_some_and(|d| d.exists()) => p,
                _ => PathBuf::from("mhaol.db"),
            }
        });

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let state = AppState::new(Some(db_path.as_path()))
        .expect("Failed to initialize database");

    state.seed_default_library();
    state.initialize_modules();

    // Start p2p-stream worker in the background
    let worker_bridge = state.worker_bridge.clone();
    tokio::spawn(async move {
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
