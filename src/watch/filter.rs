use crate::error::{Error, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;

/// Compiled pattern matcher for O(1) path matching.
///
/// This is the matcher the watch pipeline uses: patterns are compiled once when
/// the [`Watcher`](crate::Watcher) is built, so an invalid glob is reported up
/// front instead of silently matching nothing at runtime.
#[derive(Clone, Debug)]
pub struct CompiledFilter {
    glob_set: GlobSet,
}

impl CompiledFilter {
    /// Compile patterns into a `GlobSet` for fast matching.
    ///
    /// Patterns can be:
    /// - Simple names: `node_modules` matches the entry and anything inside it
    /// - Glob patterns: `*.log`, `**/*.tmp`
    /// - Directory patterns: `target/` matches everything under `target`
    ///
    /// # Errors
    ///
    /// Returns [`Error::Ignore`] naming the pattern that failed to compile.
    pub fn new(patterns: &[String]) -> Result<Self> {
        let mut builder = GlobSetBuilder::new();

        for pattern in patterns {
            for glob_pattern in Self::build_patterns(pattern) {
                let glob = Glob::new(&glob_pattern).map_err(|source| Error::Ignore {
                    pattern: Some(pattern.clone()),
                    source,
                })?;
                builder.add(glob);
            }
        }

        let glob_set = builder.build().map_err(|source| Error::Ignore {
            pattern: None,
            source,
        })?;

        Ok(Self { glob_set })
    }

    /// Build patterns for a single input (may create multiple globs)
    fn build_patterns(pattern: &str) -> Vec<String> {
        let pattern = pattern.trim();

        // If it's already a glob pattern, use as-is
        if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
            return vec![format!("**/{}", pattern)];
        }

        // Directory pattern (ends with /)
        if pattern.ends_with('/') {
            let dir_name = pattern.trim_end_matches('/');
            return vec![format!("**/{}/**", dir_name)];
        }

        // Dot-prefixed patterns (like .git, .log, .DS_Store)
        // Generate both extension match and directory match
        if pattern.starts_with('.') && !pattern.contains('/') {
            return vec![
                format!("**/*{}", pattern), // Extension: matches files ending with pattern
                format!("**/{}", pattern),  // Exact name match
                format!("**/{}/**", pattern), // Directory: matches files inside
            ];
        }

        // Simple name - match exact name and as directory
        vec![
            format!("**/{}", pattern),    // Matches the exact name
            format!("**/{}/**", pattern), // Matches files inside directory
        ]
    }

    /// Check if a path matches any ignore pattern - O(1) average case
    #[inline]
    pub fn is_ignored(&self, path: &Path) -> bool {
        self.glob_set.is_match(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiled_filter() {
        let patterns = vec![
            ".git".to_string(),
            "target/".to_string(),
            "*.log".to_string(),
            "node_modules".to_string(),
        ];

        let filter = CompiledFilter::new(&patterns).unwrap();

        // Should match
        assert!(filter.is_ignored(Path::new("/project/.git")));
        assert!(filter.is_ignored(Path::new("/project/.git/config")));
        assert!(filter.is_ignored(Path::new("/project/target/debug/binary")));
        assert!(filter.is_ignored(Path::new("/project/app.log")));
        assert!(filter.is_ignored(Path::new("/project/node_modules/package/index.js")));

        // Should not match
        assert!(!filter.is_ignored(Path::new("/project/src/main.rs")));
        assert!(!filter.is_ignored(Path::new("/project/.gitignore")));
    }

    #[test]
    fn test_compiled_filter_extensions() {
        let patterns = vec![".log".to_string(), ".tmp".to_string()];

        let filter = CompiledFilter::new(&patterns).unwrap();

        assert!(filter.is_ignored(Path::new("/project/app.log")));
        assert!(filter.is_ignored(Path::new("/project/deep/nested/file.tmp")));
        assert!(!filter.is_ignored(Path::new("/project/src/main.rs")));
    }

    #[test]
    fn test_compiled_filter_directories() {
        let patterns = vec!["build/".to_string(), "__pycache__/".to_string()];

        let filter = CompiledFilter::new(&patterns).unwrap();

        assert!(filter.is_ignored(Path::new("/project/build/output.js")));
        assert!(filter.is_ignored(Path::new("/project/src/__pycache__/module.pyc")));
        assert!(!filter.is_ignored(Path::new("/project/src/builder.rs")));
    }

    #[test]
    fn test_compiled_filter_reports_offending_pattern() {
        let err = CompiledFilter::new(&["src/**/[".to_string()]).unwrap_err();

        match err {
            Error::Ignore { pattern, .. } => assert_eq!(pattern.as_deref(), Some("src/**/[")),
            other => panic!("expected Error::Ignore, got {other:?}"),
        }
    }

    #[test]
    fn test_compiled_filter_empty_matches_nothing() {
        let filter = CompiledFilter::new(&[]).unwrap();
        assert!(!filter.is_ignored(Path::new("/project/target/debug/binary")));
    }
}
