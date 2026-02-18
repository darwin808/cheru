use super::{AppEntry, ResultType};
use plist::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

fn convert_icon_to_png(icns_path: &str, app_name: &str) -> Option<String> {
    let cache_dir = dirs::home_dir()?.join(".cache/cheru/icons");
    std::fs::create_dir_all(&cache_dir).ok()?;

    // Use a sanitized filename
    let safe_name: String = app_name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let png_path = cache_dir.join(format!("{}.png", safe_name));

    // Canonicalize and verify path is inside an app bundle
    let canonical = std::path::Path::new(icns_path).canonicalize().ok()?;
    let canonical_str = canonical.to_string_lossy();
    if !canonical_str.contains(".app/Contents/Resources/") {
        return None;
    }

    // Skip if already cached
    if png_path.exists() {
        return Some(png_path.to_string_lossy().to_string());
    }

    // Use sips to convert icns to png (128x128)
    let status = Command::new("sips")
        .args([
            "-s", "format", "png",
            "-s", "formatOptions", "best",
            "--resampleHeightWidth", "128", "128",
            icns_path,
            "--out",
            &png_path.to_string_lossy(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok()?;

    if status.success() && png_path.exists() {
        Some(png_path.to_string_lossy().to_string())
    } else {
        None
    }
}

/// Convert all .icns icons to PNG in a background-friendly way.
/// Call this from a spawned thread after startup.
pub fn convert_icons(apps: &mut [AppEntry]) {
    for app in apps.iter_mut() {
        if let Some(ref icns_path) = app.icon {
            if icns_path.ends_with(".icns") {
                app.icon = convert_icon_to_png(icns_path, &app.name);
            }
        }
    }
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
        .and_then(|icon_name| {
            let resources_dir = path.join("Contents/Resources");
            let with_ext = if icon_name.ends_with(".icns") {
                resources_dir.join(icon_name)
            } else {
                resources_dir.join(format!("{}.icns", icon_name))
            };
            if with_ext.exists() {
                return Some(with_ext.to_string_lossy().to_string());
            }
            let exact = resources_dir.join(icon_name);
            if exact.exists() {
                return Some(exact.to_string_lossy().to_string());
            }
            None
        });

    let description = dict
        .get("CFBundleGetInfoString")
        .and_then(|v| v.as_string())
        .map(|s| s.to_string());

    Some(AppEntry {
        name,
        exec,
        icon,
        description,
        result_type: ResultType::App,
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
