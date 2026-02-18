use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_autostart")]
    pub autostart: bool,
    #[serde(default)]
    pub colors: HashMap<String, String>,
}

fn default_hotkey() -> String {
    "Alt+Space".to_string()
}

fn default_theme() -> String {
    "gruvbox".to_string()
}

fn default_autostart() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: default_hotkey(),
            theme: default_theme(),
            colors: HashMap::new(),
            autostart: default_autostart(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ThemeConfig {
    pub theme: String,
    pub colors: HashMap<String, String>,
}

pub fn load() -> Config {
    let path = config_path();
    match std::fs::read_to_string(&path) {
        Ok(contents) => toml::from_str(&contents).unwrap_or_else(|e| {
            eprintln!("Warning: invalid config at {}: {}", path.display(), e);
            Config::default()
        }),
        Err(_) => {
            // Create default config file for discoverability
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let default_content = r##"# Cheru launcher configuration

# Hotkey to toggle the launcher window
# Examples: "Alt+Space", "Cmd+D", "Ctrl+Space", "Cmd+Shift+K"
hotkey = "Alt+Space"

# Theme: "gruvbox" (default), "dark", "dracula", "one-dark"
theme = "gruvbox"

# Auto-start Cheru on login (true/false)
autostart = true

# Custom color overrides (optional)
# These override any theme's colors. Use CSS color values.
# [colors]
# bg_primary = "rgba(40, 40, 40, 0.92)"
# bg_secondary = "rgba(60, 56, 54, 0.9)"
# bg_hover = "rgba(80, 73, 69, 0.8)"
# bg_selected = "rgba(215, 153, 33, 0.3)"
# text_primary = "#ebdbb2"
# text_secondary = "#a89984"
# text_placeholder = "#665c54"
# accent = "#d79921"
# border = "rgba(235, 219, 178, 0.08)"
"##;
            let _ = std::fs::write(&path, default_content);
            Config::default()
        }
    }
}

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".config")
        .join("cheru")
        .join("config.toml")
}
