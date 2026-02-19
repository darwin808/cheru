use std::process::Command;
use std::sync::{Mutex, OnceLock, RwLock};

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use thiserror::Error;

use crate::config;
use crate::indexer::{AppEntry, ResultType};
use crate::matcher::FuzzyMatcher;

pub struct AppState {
    pub index: RwLock<Vec<AppEntry>>,
    pub folder_index: OnceLock<Vec<AppEntry>>,
    pub image_index: OnceLock<Vec<AppEntry>>,
    pub matcher: Mutex<FuzzyMatcher>,
}

#[derive(Debug, Serialize)]
pub struct AppResult {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub result_type: ResultType,
}

impl From<&AppEntry> for AppResult {
    fn from(entry: &AppEntry) -> Self {
        Self {
            name: entry.name.clone(),
            exec: entry.exec.clone(),
            icon: entry.icon.clone(),
            description: entry.description.clone(),
            result_type: entry.result_type.clone(),
        }
    }
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Failed to launch application: {0}")]
    LaunchError(String),
    #[error("Window error: {0}")]
    WindowError(String),
}

impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

const MAX_RESULTS: usize = 50;

#[tauri::command]
pub fn search_apps(query: String, state: State<'_, AppState>) -> Vec<AppResult> {
    let index = state.index.read().unwrap_or_else(|e| e.into_inner());
    let mut matcher = state.matcher.lock().unwrap_or_else(|e| e.into_inner());
    let indices = matcher.search(&query, &index);

    indices
        .into_iter()
        .take(MAX_RESULTS)
        .map(|idx| AppResult::from(&index[idx]))
        .collect()
}

fn validate_exec_path(exec: &str) -> Result<(), CommandError> {
    let path = std::path::Path::new(exec);

    // Must be an absolute path
    if !path.is_absolute() {
        return Err(CommandError::LaunchError("Exec path must be absolute".to_string()));
    }

    // Canonicalize to resolve symlinks and ..
    let canonical = path.canonicalize().map_err(|e| {
        CommandError::LaunchError(format!("Cannot resolve path: {}", e))
    })?;

    let allowed_prefixes = [
        "/Applications",
        "/System/Applications",
        "/usr/bin",
        "/usr/local/bin",
        "/opt",
    ];

    // Also allow home directory Applications
    let home_apps = dirs::home_dir().map(|h| h.join("Applications"));

    let canonical_str = canonical.to_string_lossy();

    let is_allowed = allowed_prefixes.iter().any(|prefix| canonical_str.starts_with(prefix))
        || home_apps.as_ref().map_or(false, |h| canonical_str.starts_with(&h.to_string_lossy().to_string()));

    if !is_allowed {
        return Err(CommandError::LaunchError(format!(
            "Path not in allowed locations: {}",
            canonical_str
        )));
    }

    Ok(())
}

#[tauri::command]
pub fn launch_app(exec: String) -> Result<(), CommandError> {
    let exec = strip_field_codes(&exec);

    // Validate the executable path
    #[cfg(target_os = "macos")]
    {
        if exec.ends_with(".app") || exec.contains(".app/") {
            validate_exec_path(&exec)?;
            Command::new("open")
                .arg("-a")
                .arg(&exec)
                .spawn()
                .map_err(|e| CommandError::LaunchError(e.to_string()))?;
            return Ok(());
        }
    }

    let parts: Vec<&str> = exec.split_whitespace().collect();
    if parts.is_empty() {
        return Err(CommandError::LaunchError("Empty exec command".to_string()));
    }

    validate_exec_path(parts[0])?;

    Command::new(parts[0])
        .args(&parts[1..])
        .spawn()
        .map_err(|e| CommandError::LaunchError(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub fn hide_launcher_window(app: AppHandle) -> Result<(), CommandError> {
    if let Some(window) = app.get_webview_window("launcher") {
        window
            .hide()
            .map_err(|e| CommandError::WindowError(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_index_size(state: State<'_, AppState>) -> usize {
    state.index.read().unwrap_or_else(|e| e.into_inner()).len()
}

#[tauri::command]
pub fn get_theme() -> config::ThemeConfig {
    let cfg = config::load();
    config::ThemeConfig {
        theme: cfg.theme,
        colors: cfg.colors,
    }
}

#[tauri::command]
pub fn eval_expression(expr: String) -> Option<String> {
    crate::calculator::evaluate(&expr)
}

#[tauri::command]
pub fn run_system_command(id: String) -> Result<(), CommandError> {
    #[cfg(target_os = "macos")]
    {
        match id.as_str() {
            "lock" => {
                Command::new("open")
                    .arg("/System/Library/CoreServices/ScreenSaverEngine.app")
                    .spawn()
                    .map_err(|e| CommandError::LaunchError(e.to_string()))?;
            }
            "sleep" => {
                Command::new("osascript")
                    .args(["-e", "tell app \"System Events\" to sleep"])
                    .spawn()
                    .map_err(|e| CommandError::LaunchError(e.to_string()))?;
            }
            "restart" => {
                Command::new("osascript")
                    .args(["-e", "tell app \"System Events\" to restart"])
                    .spawn()
                    .map_err(|e| CommandError::LaunchError(e.to_string()))?;
            }
            "shutdown" => {
                Command::new("osascript")
                    .args(["-e", "tell app \"System Events\" to shut down"])
                    .spawn()
                    .map_err(|e| CommandError::LaunchError(e.to_string()))?;
            }
            "logout" => {
                Command::new("osascript")
                    .args(["-e", "tell app \"System Events\" to log out"])
                    .spawn()
                    .map_err(|e| CommandError::LaunchError(e.to_string()))?;
            }
            "empty-trash" => {
                Command::new("osascript")
                    .args(["-e", "tell app \"Finder\" to empty the trash"])
                    .spawn()
                    .map_err(|e| CommandError::LaunchError(e.to_string()))?;
            }
            "toggle-dark-mode" => {
                Command::new("osascript")
                    .args(["-e", "tell app \"System Events\" to tell appearance preferences to set dark mode to not dark mode"])
                    .spawn()
                    .map_err(|e| CommandError::LaunchError(e.to_string()))?;
            }
            _ => return Err(CommandError::LaunchError(format!("Unknown system command: {}", id))),
        }
    }

    #[cfg(target_os = "linux")]
    {
        match id.as_str() {
            "lock" => {
                Command::new("loginctl")
                    .arg("lock-session")
                    .spawn()
                    .map_err(|e| CommandError::LaunchError(e.to_string()))?;
            }
            "sleep" => {
                Command::new("systemctl")
                    .arg("suspend")
                    .spawn()
                    .map_err(|e| CommandError::LaunchError(e.to_string()))?;
            }
            "restart" => {
                Command::new("systemctl")
                    .arg("reboot")
                    .spawn()
                    .map_err(|e| CommandError::LaunchError(e.to_string()))?;
            }
            "shutdown" => {
                Command::new("systemctl")
                    .arg("poweroff")
                    .spawn()
                    .map_err(|e| CommandError::LaunchError(e.to_string()))?;
            }
            "logout" => {
                Command::new("loginctl")
                    .args(["terminate-user", &std::env::var("USER").unwrap_or_default()])
                    .spawn()
                    .map_err(|e| CommandError::LaunchError(e.to_string()))?;
            }
            _ => return Err(CommandError::LaunchError(format!("Unknown system command: {}", id))),
        }
    }

    Ok(())
}

#[tauri::command]
pub fn open_url(url: String) -> Result<(), CommandError> {
    if !url.starts_with("https://") {
        return Err(CommandError::LaunchError("Only HTTPS URLs allowed".into()));
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| CommandError::LaunchError(e.to_string()))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| CommandError::LaunchError(e.to_string()))?;
    }

    Ok(())
}

#[tauri::command]
pub fn search_folders(query: String, state: State<'_, AppState>) -> Vec<AppResult> {
    if query.len() < 2 {
        return Vec::new();
    }

    let folder_index = state.folder_index.get_or_init(|| {
        crate::indexer::build_folder_index()
    });

    let mut matcher = state.matcher.lock().unwrap_or_else(|e| e.into_inner());
    let indices = matcher.search(&query, folder_index);

    indices
        .into_iter()
        .take(10)
        .map(|idx| AppResult::from(&folder_index[idx]))
        .collect()
}

#[tauri::command]
pub fn search_images(query: String, state: State<'_, AppState>) -> Vec<AppResult> {
    if query.len() < 2 {
        return Vec::new();
    }

    let image_index = state.image_index.get_or_init(|| {
        crate::indexer::build_image_index()
    });

    let mut matcher = state.matcher.lock().unwrap_or_else(|e| e.into_inner());
    let indices = matcher.search(&query, image_index);

    indices
        .into_iter()
        .take(20)
        .map(|idx| AppResult::from(&image_index[idx]))
        .collect()
}

const MAX_CONTENT_RESULTS: usize = 20;

#[tauri::command]
pub fn search_file_contents(query: String) -> Vec<AppResult> {
    if query.len() < 2 {
        return Vec::new();
    }

    // Check if rg is available
    if Command::new("rg").arg("--version").output().is_err() {
        return Vec::new();
    }

    let home = dirs::home_dir().unwrap_or_default();
    let search_dirs: Vec<std::path::PathBuf> = [
        "Desktop", "Documents", "Downloads", "Projects", "Developer", "Code",
    ]
    .iter()
    .map(|d| home.join(d))
    .filter(|d| d.exists())
    .collect();

    if search_dirs.is_empty() {
        return Vec::new();
    }

    let output = Command::new("rg")
        .args([
            "--files-with-matches",
            "--max-count", "1",
            "--max-depth", "4",
            "--max-filesize", "1M",
            "--color", "never",
            "--no-heading",
            &query,
        ])
        .args(&search_dirs)
        .output();

    let output = match output {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();

    for line in stdout.lines() {
        if results.len() >= MAX_CONTENT_RESULTS {
            break;
        }
        let path = std::path::Path::new(line);
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let description = path.parent().map(|p| p.to_string_lossy().to_string());

        results.push(AppResult {
            name,
            exec: line.to_string(),
            icon: None,
            description,
            result_type: crate::indexer::ResultType::File,
        });
    }

    results
}

#[tauri::command]
pub fn open_path(path: String) -> Result<(), CommandError> {
    let p = std::path::Path::new(&path);

    // Must be absolute
    if !p.is_absolute() {
        return Err(CommandError::LaunchError("Path must be absolute".to_string()));
    }

    // Must exist
    if !p.exists() {
        return Err(CommandError::LaunchError("Path does not exist".to_string()));
    }

    // Canonicalize to resolve symlinks
    let canonical = p.canonicalize().map_err(|e| {
        CommandError::LaunchError(format!("Cannot resolve path: {}", e))
    })?;

    // Must be under home directory (not system paths)
    let home = dirs::home_dir().ok_or_else(|| {
        CommandError::LaunchError("Cannot determine home directory".to_string())
    })?;

    if !canonical.starts_with(&home) {
        return Err(CommandError::LaunchError(
            "Can only open paths under home directory".to_string(),
        ));
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(canonical.to_string_lossy().to_string())
            .spawn()
            .map_err(|e| CommandError::LaunchError(e.to_string()))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(canonical.to_string_lossy().to_string())
            .spawn()
            .map_err(|e| CommandError::LaunchError(e.to_string()))?;
    }

    Ok(())
}

#[tauri::command]
pub fn browse_directory(path: String, filter: String) -> Result<Vec<AppResult>, CommandError> {
    let dir = std::path::Path::new(&path);

    if !dir.is_absolute() {
        return Err(CommandError::LaunchError("Path must be absolute".to_string()));
    }

    if !dir.is_dir() {
        return Err(CommandError::LaunchError("Path is not a directory".to_string()));
    }

    // Canonicalize and verify under home
    let canonical = dir.canonicalize().map_err(|e| {
        CommandError::LaunchError(format!("Cannot resolve path: {}", e))
    })?;

    let home = dirs::home_dir().ok_or_else(|| {
        CommandError::LaunchError("Cannot determine home directory".to_string())
    })?;

    if !canonical.starts_with(&home) {
        return Err(CommandError::LaunchError(
            "Can only browse paths under home directory".to_string(),
        ));
    }

    let mut entries: Vec<AppEntry> = Vec::new();
    let read_dir = std::fs::read_dir(&canonical).map_err(|e| {
        CommandError::LaunchError(format!("Cannot read directory: {}", e))
    })?;

    for entry in read_dir.flatten() {
        let entry_path = entry.path();
        let name = match entry_path.file_name().and_then(|n| n.to_str()) {
            Some(n) if !n.starts_with('.') => n.to_string(),
            _ => continue,
        };

        let result_type = if entry_path.is_dir() {
            if name.ends_with(".app") {
                crate::indexer::ResultType::App
            } else {
                crate::indexer::ResultType::Folder
            }
        } else {
            let ext = entry_path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .unwrap_or_default();
            if ["png", "jpg", "jpeg", "gif", "webp", "svg"].contains(&ext.as_str()) {
                crate::indexer::ResultType::Image
            } else {
                // Skip non-image files for now — we only show folders and images in browse mode
                continue;
            }
        };

        let description = Some(canonical.to_string_lossy().to_string());

        entries.push(AppEntry {
            name,
            exec: entry_path.to_string_lossy().to_string(),
            icon: if result_type == crate::indexer::ResultType::Image {
                Some(entry_path.to_string_lossy().to_string())
            } else {
                None
            },
            description,
            result_type,
        });
    }

    // If there's a filter, fuzzy match; otherwise return all sorted
    if filter.is_empty() {
        entries.sort_by(|a, b| {
            // Folders first, then images
            let type_ord = |rt: &crate::indexer::ResultType| match rt {
                crate::indexer::ResultType::Folder => 0,
                crate::indexer::ResultType::App => 1,
                crate::indexer::ResultType::Image => 2,
                crate::indexer::ResultType::System => 3,
                crate::indexer::ResultType::File => 4,
            };
            type_ord(&a.result_type)
                .cmp(&type_ord(&b.result_type))
                .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        Ok(entries.iter().take(50).map(AppResult::from).collect())
    } else {
        let mut matcher = FuzzyMatcher::new();
        let indices = matcher.search(&filter, &entries);
        Ok(indices
            .into_iter()
            .take(50)
            .map(|idx| AppResult::from(&entries[idx]))
            .collect())
    }
}

/// Strip freedesktop field codes from exec strings (%u, %U, %f, %F, etc.)
fn strip_field_codes(exec: &str) -> String {
    exec.split_whitespace()
        .filter(|part| !part.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_field_codes() {
        assert_eq!(strip_field_codes("firefox %u"), "firefox");
        assert_eq!(strip_field_codes("code %F"), "code");
        assert_eq!(
            strip_field_codes("gimp %U --new-instance"),
            "gimp --new-instance"
        );
        assert_eq!(strip_field_codes("nautilus"), "nautilus");
    }
}
