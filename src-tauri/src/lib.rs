mod commands;
mod indexer;
mod matcher;

use commands::AppState;
use matcher::FuzzyMatcher;
use std::sync::{Mutex, RwLock};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Build app index (fast — no icon conversion yet)
            let index = indexer::build_index();
            println!("Indexed {} applications", index.len());

            // Build folder index
            let folder_index = indexer::build_folder_index();
            println!("Indexed {} folders", folder_index.len());

            // Store state
            let state = AppState {
                index: RwLock::new(index),
                folder_index,
                matcher: Mutex::new(FuzzyMatcher::new()),
            };
            app.manage(state);

            // Spawn background icon conversion
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                let state = app_handle.state::<AppState>();
                let mut index = state.index.write().unwrap();
                #[cfg(target_os = "macos")]
                {
                    indexer::macos::convert_icons(&mut index);
                    println!("Icon conversion complete");
                }
                drop(index);
            });

            // Set up system tray
            let show = MenuItemBuilder::with_id("show", "Show Launcher").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&show, &quit]).build()?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("launcher") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // Register global shortcut (Alt+Space)
            use tauri_plugin_global_shortcut::GlobalShortcutExt;
            app.global_shortcut().on_shortcut("Alt+Space", |app, _shortcut, _event| {
                if let Some(window) = app.get_webview_window("launcher") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.center();
                    }
                }
            })?;

            // macOS: hide from Dock
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            // Show window on startup in dev mode
            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("launcher") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search_apps,
            commands::launch_app,
            commands::hide_launcher_window,
            commands::get_index_size,
            commands::search_folders,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
