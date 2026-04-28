mod cloud_status;
mod frontend;

use mhaol_node::{api, load_env, AppState};
use std::path::PathBuf;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
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
                .unwrap_or_else(|_| "info,mhaol_cloud=debug,mhaol_node=debug,mhaol_torrent=debug,mhaol_p2p_stream=debug,librqbit=info".into()),
        )
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(1540);

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
            let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            if manifest_dir.exists() {
                manifest_dir.join("mhaol.db")
            } else {
                mhaol_node::default_data_dir().join("mhaol.db")
            }
        });

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let state = AppState::new(Some(db_path.as_path())).expect("Failed to initialize database");

    state.seed_default_libraries();
    state.initialize_modules();

    {
        let scan_state = state.clone();
        tokio::task::spawn_blocking(move || {
            let count = mhaol_node::api::libraries::scan_all_libraries(&scan_state);
            tracing::info!("[startup] Auto-scanned {} libraries", count);
        });
    }

    let worker_bridge = state.worker_bridge.clone();
    tokio::spawn(async move {
        worker_bridge.start().await;
    });

    {
        let recs_state = state.clone();
        tokio::spawn(async move {
            mhaol_node::recommendations_worker::run_recommendations_worker(recs_state).await;
        });
    }

    {
        let music_recs_state = state.clone();
        tokio::spawn(async move {
            mhaol_node::music_recommendations_worker::run_music_recommendations_worker(
                music_recs_state,
            )
            .await;
        });
    }

    {
        let game_recs_state = state.clone();
        tokio::spawn(async move {
            mhaol_node::game_recommendations_worker::run_game_recommendations_worker(
                game_recs_state,
            )
            .await;
        });
    }

    {
        let book_recs_state = state.clone();
        tokio::spawn(async move {
            mhaol_node::book_recommendations_worker::run_book_recommendations_worker(
                book_recs_state,
            )
            .await;
        });
    }

    state.identity_manager.ensure_identity("SIGNALING_WALLET");
    state.identity_manager.ensure_identity("CLIENT_WALLET");

    if state.identity_manager.get_profile("SIGNALING_WALLET").is_none() {
        state.identity_manager.set_profile(
            "SIGNALING_WALLET",
            &mhaol_identity::Profile {
                username: "Cloud".to_string(),
                profile_picture_url: None,
            },
        );
    }

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

    let cloud_router = axum::Router::new()
        .nest("/api/cloud", cloud_status::router())
        .with_state(state.clone());

    let app = api::build_router(state)
        .merge(cloud_router)
        .fallback(frontend::serve_frontend);

    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {}: {}", addr, e));

    tracing::info!("Cloud server (API + web UI) listening on {}", addr);

    axum::serve(listener, app).await.expect("Server error");
}
