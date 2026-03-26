use mhaol_node::{api, load_env, AppState};
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

    load_env();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,mhaol_server=debug,mhaol_torrent=debug,mhaol_p2p_stream=debug,librqbit=info".into()),
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
        .or_else(|| {
            std::env::var("DATA_DIR")
                .ok()
                .map(|d| PathBuf::from(d).join("mhaol.db"))
        })
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

    let state = AppState::new(Some(db_path.as_path())).expect("Failed to initialize database");

    state.seed_default_libraries();
    state.initialize_modules();

    // Start p2p-stream worker in the background
    let worker_bridge = state.worker_bridge.clone();
    tokio::spawn(async move {
        worker_bridge.start().await;
    });

    // Start LLM queue worker in the background
    #[cfg(not(target_os = "android"))]
    {
        let llm_state = state.clone();
        tokio::spawn(async move {
            mhaol_node::llm_worker::run_llm_worker(llm_state).await;
        });
    }

    // Start recommendations queue worker in the background
    {
        let recs_state = state.clone();
        tokio::spawn(async move {
            mhaol_node::recommendations_worker::run_recommendations_worker(recs_state).await;
        });
    }

    // Ensure dual identities: backend (SIGNALING_WALLET) + frontend (CLIENT_WALLET)
    state.identity_manager.ensure_identity("SIGNALING_WALLET");
    state.identity_manager.ensure_identity("CLIENT_WALLET");

    // Write node-defaults.json for the frontend setup modal
    {
        let local_ip = mhaol_node::api::network::get_local_ip().unwrap_or_default();
        let server_address = state
            .identity_manager
            .get_address("SIGNALING_WALLET")
            .map(|a| mhaol_identity::to_eip55_checksum(&a))
            .unwrap_or_default();
        let signaling_url = std::env::var("SIGNALING_URL")
            .unwrap_or_else(|_| "https://mhaol-signaling.project-arktosmos.partykit.dev".into());
        let defaults = serde_json::json!({
            "serverUrl": format!("http://{}:{}", local_ip, port),
            "serverAddress": server_address,
            "signalingUrl": signaling_url,
            "port": port
        });
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let json_path = std::path::PathBuf::from(manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .map(|root| root.join("apps/frontend/static/node-defaults.json"));
        if let Some(path) = json_path {
            if let Err(e) = std::fs::write(&path, serde_json::to_string_pretty(&defaults).unwrap())
            {
                tracing::warn!("Failed to write node-defaults.json: {}", e);
            } else {
                tracing::info!("Wrote node defaults to {}", path.display());
            }
        }
    }

    // Start peer service (signaling + WebRTC + catalog serving) in the background
    #[cfg(not(target_os = "android"))]
    {
        let peer_state = state.clone();
        tokio::spawn(async move {
            match mhaol_node::peer_service::PeerServiceManager::new(peer_state) {
                Ok(mut manager) => {
                    if let Err(e) = manager.start().await {
                        tracing::error!("[peer-service] Event loop error: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("[peer-service] Failed to initialize: {}", e);
                }
            }
        });
    }

    let app = api::build_router(state);

    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {}: {}", addr, e));

    tracing::info!("Backend server listening on {}", addr);

    // Optionally serve the client SPA on a separate port
    if let Ok(client_dir) = std::env::var("CLIENT_STATIC_DIR") {
        let client_port: u16 = std::env::var("CLIENT_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(1570);
        let client_index = PathBuf::from(&client_dir).join("index.html");
        let client_app = axum::Router::new().fallback_service(
            tower_http::services::ServeDir::new(&client_dir)
                .fallback(tower_http::services::ServeFile::new(client_index)),
        );
        let client_addr = format!("{}:{}", host, client_port);
        let client_listener = TcpListener::bind(&client_addr)
            .await
            .unwrap_or_else(|e| panic!("Failed to bind client to {}: {}", client_addr, e));
        tracing::info!("Client SPA listening on {}", client_addr);
        tokio::spawn(async move {
            axum::serve(client_listener, client_app)
                .await
                .expect("Client server error");
        });
    }

    axum::serve(listener, app).await.expect("Server error");
}
