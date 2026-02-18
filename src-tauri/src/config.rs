use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
}

fn default_hotkey() -> String {
    "Alt+Space".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: default_hotkey(),
        }
    }
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
            let default_content = "# Cheru launcher configuration\n# Hotkey to toggle the launcher window\n# Examples: \"Alt+Space\", \"Cmd+D\", \"Ctrl+Space\", \"Cmd+Shift+K\"\nhotkey = \"Alt+Space\"\n";
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
