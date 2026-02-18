use std::process::Command;
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use thiserror::Error;

use crate::indexer::AppEntry;
use crate::matcher::FuzzyMatcher;

pub struct AppState {
    pub index: Vec<AppEntry>,
    pub matcher: Mutex<FuzzyMatcher>,
}

#[derive(Debug, Serialize)]
pub struct AppResult {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub description: Option<String>,
}

impl From<&AppEntry> for AppResult {
    fn from(entry: &AppEntry) -> Self {
        Self {
            name: entry.name.clone(),
            exec: entry.exec.clone(),
            icon: entry.icon.clone(),
            description: entry.description.clone(),
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
    let mut matcher = state.matcher.lock().unwrap();
    let indices = matcher.search(&query, &state.index);

    indices
        .into_iter()
        .take(MAX_RESULTS)
        .map(|idx| AppResult::from(&state.index[idx]))
        .collect()
}

#[tauri::command]
pub fn launch_app(exec: String) -> Result<(), CommandError> {
    let exec = strip_field_codes(&exec);

    #[cfg(target_os = "macos")]
    {
        if exec.ends_with(".app") || exec.contains(".app/") {
            Command::new("open")
                .arg("-a")
                .arg(&exec)
                .spawn()
                .map_err(|e| CommandError::LaunchError(e.to_string()))?;
            return Ok(());
        }
    }

    // Parse command and arguments
    let parts: Vec<&str> = exec.split_whitespace().collect();
    if parts.is_empty() {
        return Err(CommandError::LaunchError("Empty exec command".to_string()));
    }

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
    state.index.len()
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
