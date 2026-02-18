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

fn setup_autostart(enabled: bool) {
    #[cfg(target_os = "macos")]
    {
        let plist_dir = dirs::home_dir()
            .unwrap_or_default()
            .join("Library/LaunchAgents");
        let plist_path = plist_dir.join("com.cheru.launcher.plist");

        if enabled {
            let _ = std::fs::create_dir_all(&plist_dir);
            let app_path = "/Applications/Cheru.app/Contents/MacOS/Cheru";
            let plist_content = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.cheru.launcher</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>"#,
                app_path
            );
            let _ = std::fs::write(&plist_path, plist_content);
        } else if plist_path.exists() {
            let _ = std::fs::remove_file(&plist_path);
        }
    }

    #[cfg(target_os = "linux")]
    {
        let autostart_dir = dirs::config_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join(".config"))
            .join("autostart");
        let desktop_path = autostart_dir.join("cheru.desktop");

        if enabled {
            let _ = std::fs::create_dir_all(&autostart_dir);
            let desktop_content = "[Desktop Entry]\nType=Application\nName=Cheru\nExec=cheru\nX-GNOME-Autostart-enabled=true\n";
            let _ = std::fs::write(&desktop_path, desktop_content);
        } else if desktop_path.exists() {
            let _ = std::fs::remove_file(&desktop_path);
        }
    }
}

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
            // Set up autostart on login
            setup_autostart(cfg.autostart);
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
            commands::get_theme,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
