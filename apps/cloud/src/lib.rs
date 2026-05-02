mod image_cache;

#[cfg(desktop)]
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    RunEvent,
};
#[cfg(desktop)]
use tauri_plugin_opener::OpenerExt;

#[cfg(desktop)]
const CLOUD_URL: &str = "http://localhost:9898";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![image_cache::image_cache_resolve])
        .setup(|app| {
            tauri::async_runtime::spawn(async move {
                mhaol_backend::run().await;
            });
            #[cfg(desktop)]
            {
                #[cfg(target_os = "macos")]
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);

                let open_item = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
                let separator = PredefinedMenuItem::separator(app)?;
                let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&open_item, &separator, &quit_item])?;
                let icon = app
                    .default_window_icon()
                    .cloned()
                    .ok_or("missing default window icon")?;
                TrayIconBuilder::with_id("mhaol-cloud-tray")
                    .tooltip("Mhaol Cloud")
                    .icon(icon)
                    .menu(&menu)
                    .show_menu_on_left_click(true)
                    .on_menu_event(|app, event| match event.id().as_ref() {
                        "open" => {
                            if let Err(e) = app.opener().open_url(CLOUD_URL, None::<&str>) {
                                log::error!("failed to open {CLOUD_URL}: {e}");
                            }
                        }
                        "quit" => app.exit(0),
                        _ => {}
                    })
                    .build(app)?;
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app, event| {
        #[cfg(desktop)]
        if let RunEvent::ExitRequested { api, code, .. } = &event {
            // No windows are ever created — keep the process alive so the tray stays.
            // Only the tray's Quit item (which calls app.exit(0)) actually quits.
            if code.is_none() {
                api.prevent_exit();
            }
        }
    });
}
