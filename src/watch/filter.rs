use std::path::Path;

pub fn is_ignored<P: AsRef<Path>>(path: P, ignored_patterns: &[String]) -> bool {
    let path_str = path.as_ref().to_str().unwrap_or("");

    for pattern in ignored_patterns {
        if path_str.ends_with(pattern) {
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
        ];

        assert!(is_ignored("path/to/.git", &ignored_patterns));
        assert!(is_ignored("some/directory/.DS_Store", &ignored_patterns));
        assert!(is_ignored("project/target/", &ignored_patterns));
        assert!(!is_ignored("normal/file.txt", &ignored_patterns));
        assert!(!is_ignored("another/directory", &ignored_patterns));
    }
}
