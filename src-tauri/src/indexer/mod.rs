use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResultType {
    App,
    Folder,
    Image,
    System,
}

const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "svg"];
const MAX_IMAGES: usize = 2000;
const MAX_FOLDERS: usize = 500;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub result_type: ResultType,
}

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

pub fn build_index() -> Vec<AppEntry> {
    #[cfg(target_os = "linux")]
    {
        linux::index_apps()
    }
    #[cfg(target_os = "macos")]
    {
        macos::index_apps()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Vec::new()
    }
}

pub fn build_folder_index() -> Vec<AppEntry> {
    let mut folders = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let home = dirs::home_dir().unwrap_or_default();
    let search_dirs = vec![
        home.join("Desktop"),
        home.join("Documents"),
        home.join("Downloads"),
        home.join("Pictures"),
        home.join("Music"),
        home.join("Movies"),
        home.join("Projects"),
        home.join("Developer"),
        home.join("Code"),
    ];

    for dir in &search_dirs {
        if folders.len() >= MAX_FOLDERS {
            break;
        }
        collect_folders(dir, 0, 2, &mut folders, &mut seen);
    }

    folders.truncate(MAX_FOLDERS);
    folders.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    folders
}

pub fn build_image_index() -> Vec<AppEntry> {
    let mut images = Vec::new();
    let mut seen = std::collections::HashSet::new();

    let home = dirs::home_dir().unwrap_or_default();
    let search_dirs = vec![
        home.join("Desktop"),
        home.join("Documents"),
        home.join("Downloads"),
        home.join("Pictures"),
        home.join("Projects"),
        home.join("Developer"),
        home.join("Code"),
    ];

    for dir in &search_dirs {
        if images.len() >= MAX_IMAGES {
            break;
        }
        collect_images(dir, 0, 2, &mut images, &mut seen);
    }

    images.truncate(MAX_IMAGES);
    images.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    images
}

pub fn build_system_commands() -> Vec<AppEntry> {
    let mut cmds = Vec::new();

    #[cfg(target_os = "macos")]
    {
        let commands = [
            ("Lock Screen", "system:lock", "Lock the screen"),
            ("Sleep", "system:sleep", "Put the computer to sleep"),
            ("Restart", "system:restart", "Restart the computer"),
            ("Shut Down", "system:shutdown", "Shut down the computer"),
            ("Log Out", "system:logout", "Log out of the current session"),
            ("Empty Trash", "system:empty-trash", "Empty the Trash"),
            ("Toggle Dark Mode", "system:toggle-dark-mode", "Switch between light and dark mode"),
        ];
        for (name, exec, desc) in commands {
            cmds.push(AppEntry {
                name: name.to_string(),
                exec: exec.to_string(),
                icon: None,
                description: Some(desc.to_string()),
                result_type: ResultType::System,
            });
        }
    }

    #[cfg(target_os = "linux")]
    {
        let commands = [
            ("Lock Screen", "system:lock", "Lock the screen"),
            ("Sleep", "system:sleep", "Suspend the computer"),
            ("Restart", "system:restart", "Restart the computer"),
            ("Shut Down", "system:shutdown", "Shut down the computer"),
            ("Log Out", "system:logout", "Log out of the current session"),
        ];
        for (name, exec, desc) in commands {
            cmds.push(AppEntry {
                name: name.to_string(),
                exec: exec.to_string(),
                icon: None,
                description: Some(desc.to_string()),
                result_type: ResultType::System,
            });
        }
    }

    cmds
}

fn collect_images(
    dir: &std::path::Path,
    depth: usize,
    max_depth: usize,
    images: &mut Vec<AppEntry>,
    seen: &mut std::collections::HashSet<std::path::PathBuf>,
) {
    if depth >= max_depth || images.len() >= MAX_IMAGES {
        return;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // Skip hidden entries
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) if !n.starts_with('.') => n.to_string(),
            _ => continue,
        };

        if path.is_dir() {
            // Skip noise directories
            if matches!(
                name.as_str(),
                "node_modules"
                    | "target"
                    | "build"
                    | "dist"
                    | "__pycache__"
                    | ".git"
                    | "Library"
                    | "Caches"
                    | "Containers"
                    | "HTTPStorages"
                    | "WebKit"
                    | "Saved Application State"
                    | "Application Support"
                    | "Application Scripts"
                    | "Group Containers"
                    | "GPUCache"
                    | "DerivedData"
                    | "Logs"
                    | "tmp"
                    | "var"
                    | "usr"
            ) {
                continue;
            }
            if name.ends_with(".app") {
                continue;
            }
            collect_images(&path, depth + 1, max_depth, images, seen);
        } else if path.is_file() {
            // Check if it's an image
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());

            if let Some(ref ext) = ext {
                if IMAGE_EXTENSIONS.contains(&ext.as_str()) && seen.insert(path.clone()) {
                    let description = path.parent().map(|p| p.to_string_lossy().to_string());
                    images.push(AppEntry {
                        name,
                        exec: path.to_string_lossy().to_string(),
                        icon: Some(path.to_string_lossy().to_string()), // icon IS the image itself
                        description,
                        result_type: ResultType::Image,
                    });
                }
            }
        }
    }
}

fn collect_folders(
    dir: &std::path::Path,
    depth: usize,
    max_depth: usize,
    folders: &mut Vec<AppEntry>,
    seen: &mut std::collections::HashSet<std::path::PathBuf>,
) {
    if depth >= max_depth || folders.len() >= MAX_FOLDERS {
        return;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) if !n.starts_with('.') => n.to_string(),
            _ => continue,
        };

        if matches!(
            name.as_str(),
            "node_modules"
                | "target"
                | "build"
                | "dist"
                | "__pycache__"
                | ".git"
                | "Library"
                | "Caches"
                | "Containers"
                | "HTTPStorages"
                | "WebKit"
                | "Saved Application State"
                | "Application Support"
                | "Application Scripts"
                | "Group Containers"
                | "GPUCache"
                | "DerivedData"
                | "Logs"
                | "tmp"
                | "var"
                | "usr"
        ) {
            continue;
        }

        if name.ends_with(".app") {
            continue;
        }

        if !seen.insert(path.clone()) {
            continue;
        }

        let description = path.parent().map(|p| p.to_string_lossy().to_string());

        folders.push(AppEntry {
            name,
            exec: path.to_string_lossy().to_string(),
            icon: None,
            description,
            result_type: ResultType::Folder,
        });

        collect_folders(&path, depth + 1, max_depth, folders, seen);
    }
}
