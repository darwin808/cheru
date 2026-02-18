use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResultType {
    App,
    Folder,
}

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
        home.clone(),
        home.join("Desktop"),
        home.join("Documents"),
        home.join("Downloads"),
        home.join("Projects"),
        home.join("Developer"),
        home.join("Code"),
    ];

    for dir in &search_dirs {
        collect_folders(dir, 0, 3, &mut folders, &mut seen);
    }

    folders.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    folders
}

fn collect_folders(
    dir: &std::path::Path,
    depth: usize,
    max_depth: usize,
    folders: &mut Vec<AppEntry>,
    seen: &mut std::collections::HashSet<std::path::PathBuf>,
) {
    if depth >= max_depth {
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
            "node_modules" | "target" | "build" | "dist" | "__pycache__" | ".git"
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
