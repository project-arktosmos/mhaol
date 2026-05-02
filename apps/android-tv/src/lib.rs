// Mhaol Android TV — Tauri shell that hosts the SPA in its own WebView.
// Does NOT embed the backend. The user must point the SPA at a reachable
// Mhaol backend via the in-app Settings page (defaults to
// `http://127.0.0.1:9898`, which on the Android emulator only works when
// `cargo tauri android dev --host` is forwarding the host's localhost).

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running mhaol-android-tv");
}
