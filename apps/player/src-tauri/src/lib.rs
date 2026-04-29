mod image_cache;
#[cfg(not(target_os = "android"))]
mod ytdl_server;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(not(target_os = "android"))]
    ytdl_server::spawn();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![image_cache::image_cache_resolve])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
