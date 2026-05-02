// Mhaol Mobile — Tauri shell that hosts the SPA in its own WebView and
// boots the embedded backend (`mhaol_backend::run()`) inside the same
// process. The SPA defaults to `http://127.0.0.1:9898`, which is exactly
// where the embedded backend binds.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|_app| {
            tauri::async_runtime::spawn(async move {
                mhaol_backend::run().await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running mhaol-android-mobile");
}
