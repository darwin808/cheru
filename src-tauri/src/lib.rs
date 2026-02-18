mod commands;
mod config;
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

            // Build image index
            let image_index = indexer::build_image_index();
            println!("Indexed {} images", image_index.len());

            // Store state
            let state = AppState {
                index: RwLock::new(index),
                folder_index,
                image_index,
                matcher: Mutex::new(FuzzyMatcher::new()),
            };
            app.manage(state);

            // Spawn background icon conversion (macOS only)
            #[cfg(target_os = "macos")]
            {
                let app_handle = app.handle().clone();
                std::thread::spawn(move || {
                    let state = app_handle.state::<AppState>();
                    let mut index = state.index.write().unwrap();
                    indexer::macos::convert_icons(&mut index);
                    println!("Icon conversion complete");
                });
            }

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

            // Register global shortcut from config
            let cfg = config::load();
            println!("Hotkey: {}", cfg.hotkey);
            use tauri_plugin_global_shortcut::GlobalShortcutExt;
            app.global_shortcut().on_shortcut(cfg.hotkey.as_str(), |app, _shortcut, event| {
                if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    if let Some(window) = app.get_webview_window("launcher") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let _ = window.center();
                        }
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
            commands::search_images,
            commands::open_path,
            commands::browse_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
