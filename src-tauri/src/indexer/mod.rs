use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub description: Option<String>,
}

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

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
