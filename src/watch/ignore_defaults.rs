/// Default ignore patterns for common development environments
pub const DEFAULT_IGNORE_PATTERNS: &[&str] = &[
    // Version Control
    ".git/",
    ".hg/",
    ".svn/",
    // Rust
    "target/",
    // Node.js / JavaScript
    "node_modules/",
    ".npm/",
    ".next/",
    ".nuxt/",
    "dist/",
    // Python
    "__pycache__/",
    ".venv/",
    "venv/",
    ".pyc",
    ".pyo",
    ".egg-info/",
    // Go
    "vendor/",
    // IDE / Editors
    ".idea/",
    ".vscode/",
    ".swp",
    ".swo",
    ".DS_Store",
    // Build outputs
    "build/",
    "out/",
    ".o",
    ".a",
    ".so",
    ".dylib",
    // Logs and temp files
    ".log",
    ".tmp",
    ".temp",
    ".cache/",
    // Coverage
    "coverage/",
    ".nyc_output/",
    "htmlcov/",
];

/// Convert default patterns to owned strings
pub fn default_ignore_patterns() -> Vec<String> {
    DEFAULT_IGNORE_PATTERNS
        .iter()
        .map(|s| (*s).to_string())
        .collect()
}

/// Merge user patterns with defaults
/// If user provides patterns, use those; otherwise use defaults
pub fn merge_with_defaults(user_patterns: Option<Vec<String>>) -> Vec<String> {
    match user_patterns {
        Some(patterns) if !patterns.is_empty() => patterns,
        _ => default_ignore_patterns(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_patterns_not_empty() {
        let patterns = default_ignore_patterns();
        assert!(!patterns.is_empty());
        assert!(patterns.contains(&".git/".to_string()));
        assert!(patterns.contains(&"target/".to_string()));
        assert!(patterns.contains(&"node_modules/".to_string()));
    }

    #[test]
    fn test_merge_with_user_patterns() {
        let user = Some(vec!["custom/".to_string()]);
        let result = merge_with_defaults(user);
        assert_eq!(result, vec!["custom/".to_string()]);
    }

    #[test]
    fn test_merge_with_empty_uses_defaults() {
        let result = merge_with_defaults(None);
        assert!(!result.is_empty());
        assert!(result.contains(&".git/".to_string()));
    }

    #[test]
    fn test_merge_with_empty_vec_uses_defaults() {
        let result = merge_with_defaults(Some(vec![]));
        assert!(!result.is_empty());
    }
}
