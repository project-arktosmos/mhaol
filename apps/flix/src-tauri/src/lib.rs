use tauri::Manager;

const SERVER_HOST: &str = "127.0.0.1";

fn setup_backend_on_port(app: &tauri::App, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", SERVER_HOST, port);

    let std_listener = std::net::TcpListener::bind(&addr)
        .map_err(|e| format!("Port {} is not available: {}", port, e))?;
    std_listener.set_nonblocking(true)?;

    let db_path = {
        let app_dir = app
            .path()
            .app_data_dir()
            .expect("failed to resolve app data directory");
        std::fs::create_dir_all(&app_dir).expect("failed to create app data directory");
        app_dir.join("mhaol.db")
    };

    tauri::async_runtime::spawn(async move {
        let state = mhaol_backend::AppState::new(Some(db_path.as_path()))
            .expect("failed to initialize backend");
        state.seed_default_library();
        state.initialize_modules();
        let router = mhaol_backend::api::build_router(state);
        let listener = tokio::net::TcpListener::from_std(std_listener)
            .expect("failed to convert listener");
        axum::serve(listener, router)
            .await
            .expect("backend server error");
    });

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    mhaol_backend::load_env_app();
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            if setup_backend_on_port(app, 1530).is_err() {
                setup_backend_on_port(app, 1531)?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
