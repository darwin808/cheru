use super::{AppEntry, ResultType};
use freedesktop_desktop_entry::{DesktopEntry, Iter as DesktopIter};
use std::collections::HashSet;
use std::fs;

pub fn index_apps() -> Vec<AppEntry> {
    let mut apps = Vec::new();
    let mut seen = HashSet::new();

    for path in DesktopIter::new(freedesktop_desktop_entry::default_paths()) {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(entry) = DesktopEntry::from_str(&path, &content, &["en"]) {
                // Skip non-application types
                if entry.type_() != Some("Application") {
                    continue;
                }

                // Skip hidden and no-display entries
                if entry.no_display() || entry.hidden() {
                    continue;
                }

                let name = match entry.name(&["en"]) {
                    Some(n) => n.to_string(),
                    None => continue,
                };

                // Deduplicate by name
                if !seen.insert(name.clone()) {
                    continue;
                }

                let exec = match entry.exec() {
                    Some(e) => e.to_string(),
                    None => continue,
                };

                apps.push(AppEntry {
                    name,
                    exec,
                    icon: entry.icon().map(|s| s.to_string()),
                    description: entry.comment(&["en"]).map(|s| s.to_string()),
                    result_type: ResultType::App,
                });
            }
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_apps_returns_vec() {
        let apps = index_apps();
        // On a Linux system with desktop entries, this should return some apps
        // On CI/macOS, it returns empty which is fine
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
}
