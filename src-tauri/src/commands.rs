use std::process::Command;
use std::sync::{Mutex, RwLock};

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use thiserror::Error;

use crate::indexer::{AppEntry, ResultType};
use crate::matcher::FuzzyMatcher;

pub struct AppState {
    pub index: RwLock<Vec<AppEntry>>,
    pub folder_index: Vec<AppEntry>,
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
pub fn search_folders(query: String, state: State<'_, AppState>) -> Vec<AppResult> {
    if query.len() < 2 {
        return Vec::new();
    }

    let mut matcher = state.matcher.lock().unwrap_or_else(|e| e.into_inner());
    let indices = matcher.search(&query, &state.folder_index);

    indices
        .into_iter()
        .take(10)
        .map(|idx| AppResult::from(&state.folder_index[idx]))
        .collect()
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
