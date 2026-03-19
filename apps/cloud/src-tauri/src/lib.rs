#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    mhaol_tauri_core::load_env();
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            mhaol_tauri_core::setup_backend(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
