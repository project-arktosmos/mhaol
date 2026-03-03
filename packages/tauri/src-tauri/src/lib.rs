use tauri::Manager;

const SERVER_PORT: u16 = 1530;
const SERVER_HOST: &str = "127.0.0.1";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    mhaol_backend::load_env_app();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Determine database path
            let db_path = {
                let app_dir = app
                    .path()
                    .app_data_dir()
                    .expect("failed to resolve app data directory");
                std::fs::create_dir_all(&app_dir)
                    .expect("failed to create app data directory");
                app_dir.join("mhaol.db")
            };

            // Start the Rust backend server in a background task
            tauri::async_runtime::spawn(async move {
                let state = mhaol_backend::AppState::new(Some(db_path.as_path()))
                    .expect("failed to initialize backend");
                state.seed_default_library();
                state.initialize_modules();
                let router = mhaol_backend::api::build_router(state);
                let addr = format!("{}:{}", SERVER_HOST, SERVER_PORT);
                let listener = tokio::net::TcpListener::bind(&addr)
                    .await
                    .expect("failed to bind backend server");
                axum::serve(listener, router)
                    .await
                    .expect("backend server error");
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
