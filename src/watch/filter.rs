use std::path::Path;

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
}
