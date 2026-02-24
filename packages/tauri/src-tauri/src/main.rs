#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::TcpStream;
use std::process::{Child, Command};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::Manager;

const SERVER_PORT: u16 = 1530;
const SERVER_HOST: &str = "127.0.0.1";

struct ServerProcess(Mutex<Option<Child>>);

impl Drop for ServerProcess {
    fn drop(&mut self) {
        if let Some(mut child) = self.0.lock().unwrap().take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

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

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            #[cfg(not(debug_assertions))]
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

                app.manage(ServerProcess(Mutex::new(Some(child))));

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
