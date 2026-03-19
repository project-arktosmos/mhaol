use std::sync::Arc;

const SERVER_PORT: u16 = 1530;
const SERVER_HOST: &str = "127.0.0.1";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|_app| {
            setup_torrent_server();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_torrent_server() {
    tauri::async_runtime::spawn(async move {
        let download_dir = std::env::var("TORRENT_DOWNLOAD_DIR").unwrap_or_else(|_| {
            std::env::var("HOME")
                .map(|home| format!("{}/Downloads/torrents", home))
                .unwrap_or_else(|_| "/tmp/torrents".to_string())
        });

        let manager = Arc::new(mhaol_torrent::TorrentManager::new());

        let config = mhaol_torrent::TorrentConfig {
            download_path: std::path::PathBuf::from(&download_dir),
            http_api_bind_addr: None,
            ..Default::default()
        };

        if let Err(e) = manager.initialize(config).await {
            log::error!("Failed to initialize torrent manager: {}", e);
            return;
        }

        log::info!("Torrent manager initialized, download dir: {}", download_dir);

        let cors = tower_http::cors::CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any);

        let app = axum::Router::new()
            .nest("/api/torrent", mhaol_torrent::api::router())
            .layer(cors)
            .with_state(manager);

        let addr = format!("{}:{}", SERVER_HOST, SERVER_PORT);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .expect("failed to bind torrent server");

        log::info!("Torrent server listening on {}", addr);

        axum::serve(listener, app)
            .await
            .expect("torrent server error");
    });
}
