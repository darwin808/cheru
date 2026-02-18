use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use nucleo_matcher::{Config, Matcher, Utf32Str};

use crate::indexer::AppEntry;

pub struct FuzzyMatcher {
    matcher: Matcher,
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        Self {
            matcher: Matcher::new(Config::DEFAULT.match_paths()),
        }
    }

    /// Search apps by query. Returns indices into the apps slice, sorted by score descending.
    /// Empty query returns all indices in alphabetical order (apps are pre-sorted).
    pub fn search(&mut self, query: &str, apps: &[AppEntry]) -> Vec<usize> {
        if query.is_empty() {
            return (0..apps.len()).collect();
        }

        let atom = Atom::new(
            query,
            CaseMatching::Smart,
            Normalization::Smart,
            AtomKind::Fuzzy,
            false,
        );

        let mut buf = Vec::new();
        let mut scored: Vec<(usize, u16)> = apps
            .iter()
            .enumerate()
            .filter_map(|(idx, app)| {
                let haystack = Utf32Str::new(&app.name, &mut buf);
                let score = atom.score(haystack, &mut self.matcher)?;
                Some((idx, score))
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.into_iter().map(|(idx, _)| idx).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::AppEntry;

    fn make_app(name: &str) -> AppEntry {
        AppEntry {
            name: name.to_string(),
            exec: format!("/usr/bin/{}", name.to_lowercase()),
            icon: None,
            description: None,
            result_type: crate::indexer::ResultType::App,
        }
    }

    #[test]
    fn test_empty_query_returns_all() {
        let apps = vec![make_app("Alpha"), make_app("Beta"), make_app("Charlie")];
        let mut matcher = FuzzyMatcher::new();
        let results = matcher.search("", &apps);
        assert_eq!(results.len(), 3);
        assert_eq!(results, vec![0, 1, 2]);
    }

    #[test]
    fn test_exact_match_scores_highest() {
        let apps = vec![
            make_app("Firefox"),
            make_app("Files"),
            make_app("Finder"),
        ];
        let mut matcher = FuzzyMatcher::new();
        let results = matcher.search("Firefox", &apps);
        assert!(!results.is_empty());
        assert_eq!(results[0], 0); // Firefox should be first
    }

    #[test]
    fn test_no_match_returns_empty() {
        let apps = vec![make_app("Firefox"), make_app("Chrome")];
        let mut matcher = FuzzyMatcher::new();
        let results = matcher.search("zzzzz", &apps);
        assert!(results.is_empty());
    }

    #[test]
    fn test_fuzzy_matching() {
        let apps = vec![
            make_app("Visual Studio Code"),
            make_app("Vim"),
            make_app("VLC"),
        ];
        let mut matcher = FuzzyMatcher::new();
        let results = matcher.search("vsc", &apps);
        // "vsc" should match "Visual Studio Code"
        assert!(results.contains(&0));
    }

    #[test]
    fn test_case_insensitive() {
        let apps = vec![make_app("Firefox")];
        let mut matcher = FuzzyMatcher::new();
        let results = matcher.search("firefox", &apps);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 0);
    }
}
