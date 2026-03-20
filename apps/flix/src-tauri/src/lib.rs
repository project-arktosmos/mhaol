use tauri::Manager;

const SERVER_HOST: &str = "127.0.0.1";

fn setup_backend_on_port(app: &tauri::App, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", SERVER_HOST, port);
    eprintln!("[flix] binding backend to {}", addr);

    let std_listener = std::net::TcpListener::bind(&addr)
        .map_err(|e| {
            eprintln!("[flix] failed to bind {}: {}", addr, e);
            format!("Port {} is not available: {}", port, e)
        })?;
    std_listener.set_nonblocking(true)?;

    let db_path = {
        let app_dir = app
            .path()
            .app_data_dir()
            .expect("failed to resolve app data directory");
        std::fs::create_dir_all(&app_dir).expect("failed to create app data directory");
        app_dir.join("mhaol.db")
    };
    eprintln!("[flix] db path: {:?}", db_path);

    tauri::async_runtime::spawn(async move {
        eprintln!("[flix] initializing AppState...");
        let state = match mhaol_backend::AppState::new(Some(db_path.as_path())) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[flix] AppState::new failed: {}", e);
                return;
            }
        };
        eprintln!("[flix] seeding default library...");
        state.seed_default_library();
        eprintln!("[flix] initializing modules...");
        state.initialize_modules();
        let router = mhaol_backend::api::build_router(state);
        let listener = tokio::net::TcpListener::from_std(std_listener)
            .expect("failed to convert listener");
        eprintln!("[flix] backend serving on port {}", port);
        axum::serve(listener, router)
            .await
            .expect("backend server error");
    });

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    mhaol_backend::load_env_app();
    // Fallback: bake API keys at compile time for Android where .env.app is unavailable
    if std::env::var("TMDB_API_KEY").is_err() {
        std::env::set_var("TMDB_API_KEY", env!("TMDB_API_KEY"));
    }
    if std::env::var("TMDB_READ_ACCESS_TOKEN").is_err() {
        std::env::set_var("TMDB_READ_ACCESS_TOKEN", env!("TMDB_READ_ACCESS_TOKEN"));
    }
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
