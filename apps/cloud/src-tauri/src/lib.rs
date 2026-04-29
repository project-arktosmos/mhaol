mod image_cache;

#[cfg(desktop)]
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};
#[cfg(desktop)]
use tauri_plugin_opener::OpenerExt;

#[cfg(desktop)]
const CLOUD_URL: &str = "http://localhost:9898";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![image_cache::image_cache_resolve])
        .setup(|app| {
            #[cfg(desktop)]
            {
                let open_item = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&open_item])?;
                let icon = app
                    .default_window_icon()
                    .cloned()
                    .ok_or("missing default window icon")?;
                TrayIconBuilder::with_id("mhaol-cloud-tray")
                    .tooltip("Mhaol Cloud")
                    .icon(icon)
                    .menu(&menu)
                    .show_menu_on_left_click(true)
                    .on_menu_event(|app, event| {
                        if event.id().as_ref() == "open" {
                            if let Err(e) = app.opener().open_url(CLOUD_URL, None::<&str>) {
                                log::error!("failed to open {CLOUD_URL}: {e}");
                            }
                        }
                    })
                    .build(app)?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
