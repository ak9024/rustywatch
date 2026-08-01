use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;

/// Compiled pattern matcher for O(1) path matching
#[derive(Clone)]
pub struct CompiledFilter {
    glob_set: GlobSet,
}

impl CompiledFilter {
    /// Compile patterns into a GlobSet for fast matching
    /// Patterns can be:
    /// - Simple names: "node_modules" matches any path containing it
    /// - Glob patterns: "*.log", "**/*.tmp"
    /// - Directory patterns: "target/" matches target directory
    pub fn new(patterns: &[String]) -> Result<Self, globset::Error> {
        let mut builder = GlobSetBuilder::new();

        for pattern in patterns {
            for glob_pattern in Self::build_patterns(pattern) {
                let glob = Glob::new(&glob_pattern)?;
                builder.add(glob);
            }
        }

        let glob_set = builder.build()?;
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

/// Legacy function for backward compatibility
pub fn is_ignored<P: AsRef<Path>>(path: P, ignored_patterns: &[String]) -> bool {
    let path = path.as_ref();
    let path_str = path.to_str().unwrap_or("");

    for pattern in ignored_patterns {
        if path_str.ends_with(pattern) || (pattern.ends_with('/') && path_str.contains(pattern)) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ignored() {
        let ignored_patterns = vec![
            String::from(".git"),
            String::from(".DS_Store"),
            String::from("target/"),
            String::from("node_modules"),
        ];

        // Test exact matches
        assert!(is_ignored("path/to/.git", &ignored_patterns));
        assert!(is_ignored("some/directory/.DS_Store", &ignored_patterns));
        assert!(is_ignored("project/target/", &ignored_patterns));
        assert!(is_ignored("node_modules", &ignored_patterns));

        // Test directory patterns
        assert!(is_ignored("project/target/debug", &ignored_patterns));
        assert!(is_ignored("nested/path/target/release", &ignored_patterns));

        // Test non-matches
        assert!(!is_ignored("normal/file.txt", &ignored_patterns));
        assert!(!is_ignored("another/directory", &ignored_patterns));
        assert!(!is_ignored(".gitignore", &ignored_patterns));
        assert!(!is_ignored("targets", &ignored_patterns));

        // Test empty path
        assert!(!is_ignored("", &ignored_patterns));

        // Test case sensitivity
        assert!(!is_ignored("path/to/.GIT", &ignored_patterns));
        assert!(!is_ignored("some/directory/.ds_store", &ignored_patterns));
    }

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
}
