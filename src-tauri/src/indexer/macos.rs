use super::AppEntry;
use plist::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn index_apps() -> Vec<AppEntry> {
    let mut apps = Vec::new();
    let mut seen = HashSet::new();

    let search_dirs = get_search_dirs();

    for dir in &search_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("app") {
                    if let Some(app) = parse_app_bundle(&path) {
                        if seen.insert(app.name.clone()) {
                            apps.push(app);
                        }
                    }
                }
            }
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

fn get_search_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications"),
        PathBuf::from("/System/Applications/Utilities"),
    ];

    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join("Applications"));
    }

    dirs
}

fn parse_app_bundle(path: &Path) -> Option<AppEntry> {
    let plist_path = path.join("Contents/Info.plist");
    let plist = Value::from_file(&plist_path).ok()?;
    let dict = plist.as_dictionary()?;

    let name = dict
        .get("CFBundleDisplayName")
        .or_else(|| dict.get("CFBundleName"))
        .and_then(|v| v.as_string())
        .map(|s| s.to_string())
        .or_else(|| {
            // Fall back to the .app folder name
            path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })?;

    let exec = path.to_string_lossy().to_string();

    let icon = dict
        .get("CFBundleIconFile")
        .and_then(|v| v.as_string())
        .map(|s| s.to_string());

    let description = dict
        .get("CFBundleGetInfoString")
        .and_then(|v| v.as_string())
        .map(|s| s.to_string());

    Some(AppEntry {
        name,
        exec,
        icon,
        description,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_apps_returns_vec() {
        let apps = index_apps();
        // On macOS, should find at least some system apps
        // May be empty on Linux CI
        assert!(apps.iter().all(|a| !a.name.is_empty()));
        assert!(apps.iter().all(|a| !a.exec.is_empty()));
    }

    #[test]
    fn test_index_apps_sorted() {
        let apps = index_apps();
        for window in apps.windows(2) {
            assert!(window[0].name.to_lowercase() <= window[1].name.to_lowercase());
        }
    }

    #[test]
    fn test_index_apps_no_duplicates() {
        let apps = index_apps();
        let mut names = HashSet::new();
        for app in &apps {
            assert!(names.insert(&app.name), "Duplicate app: {}", app.name);
        }
    }

    #[test]
    fn test_get_search_dirs() {
        let dirs = get_search_dirs();
        assert!(dirs.contains(&PathBuf::from("/Applications")));
        assert!(dirs.contains(&PathBuf::from("/System/Applications")));
    }
}
