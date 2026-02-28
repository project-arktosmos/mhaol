mod commands;

#[cfg(not(target_os = "android"))]
use std::net::TcpStream;
#[cfg(not(target_os = "android"))]
use std::process::{Child, Command};
#[cfg(not(target_os = "android"))]
use std::sync::Mutex as StdMutex;
#[cfg(not(target_os = "android"))]
use std::time::{Duration, Instant};

use std::sync::Mutex;

use rusqlite::Connection;
use tauri::Manager;

use commands::db::{self, AppDb};
use commands::search;

#[cfg(not(target_os = "android"))]
const SERVER_PORT: u16 = 1530;
#[cfg(not(target_os = "android"))]
const SERVER_HOST: &str = "127.0.0.1";

#[cfg(not(target_os = "android"))]
struct ServerProcess(StdMutex<Option<Child>>);

#[cfg(not(target_os = "android"))]
impl Drop for ServerProcess {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.lock().unwrap().take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[cfg(not(target_os = "android"))]
fn wait_for_server(host: &str, port: u16, timeout: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if TcpStream::connect(format!("{}:{}", host, port)).is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            db::get_libraries,
            db::add_library,
            db::get_library_items,
            db::get_media_types,
            db::get_categories,
            db::get_settings,
            db::set_setting,
            db::update_item_category,
            db::browse_directory,
            db::scan_library,
            search::search_torrents,
        ])
        .setup(|app| {
            // Initialize SQLite database for mobile (on desktop the Node.js server handles this)
            #[cfg(target_os = "android")]
            {
                let app_dir = app
                    .path()
                    .app_data_dir()
                    .expect("failed to resolve app data directory");
                std::fs::create_dir_all(&app_dir).expect("failed to create app data directory");
                let db_path = app_dir.join("mhaol.db");
                let conn =
                    Connection::open(&db_path).expect("failed to open SQLite database");
                db::initialize_db(&conn).expect("failed to initialize database schema");
                app.manage(AppDb(Mutex::new(conn)));
            }

            // On desktop: spawn the Node.js server in release mode
            #[cfg(all(not(debug_assertions), not(target_os = "android")))]
            {
                let resource_dir = app
                    .path()
                    .resource_dir()
                    .expect("failed to resolve resource directory");
                let server_dir = resource_dir.join("server");

                let child = Command::new("node")
                    .arg("index.js")
                    .current_dir(&server_dir)
                    .env("PORT", SERVER_PORT.to_string())
                    .env("HOST", SERVER_HOST)
                    .spawn()
                    .expect("failed to start Node.js server — is Node.js installed?");

                app.manage(ServerProcess(StdMutex::new(Some(child))));

                if !wait_for_server(SERVER_HOST, SERVER_PORT, Duration::from_secs(15)) {
                    panic!("server failed to start within 15 seconds");
                }

                let url = format!("http://{}:{}", SERVER_HOST, SERVER_PORT);
                let window = app
                    .get_webview_window("main")
                    .expect("failed to get main window");
                window.eval(&format!("window.location.replace('{}')", url))?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
