use super::templates::ProjectType;
use std::path::Path;

/// Detects the project type based on marker files in the directory
pub fn detect_project_type(dir: &Path) -> Option<ProjectType> {
    if dir.join("Cargo.toml").exists() {
        Some(ProjectType::Rust)
    } else if dir.join("go.mod").exists() {
        Some(ProjectType::Go)
    } else if dir.join("bun.lockb").exists() {
        Some(ProjectType::Bun)
    } else if dir.join("package.json").exists() {
        Some(ProjectType::NodeJS)
    } else if dir.join("pyproject.toml").exists()
        || dir.join("setup.py").exists()
        || dir.join("requirements.txt").exists()
    {
        Some(ProjectType::Python)
    } else {
        None
    }
}

/// Tries to extract the project name from configuration files
pub fn detect_project_name(dir: &Path) -> Option<String> {
    // Try Cargo.toml
    if let Ok(content) = std::fs::read_to_string(dir.join("Cargo.toml")) {
        for line in content.lines() {
            if line.starts_with("name") {
                if let Some(name) = line.split('=').nth(1) {
                    let name = name.trim().trim_matches('"').trim_matches('\'');
                    return Some(name.to_string());
                }
            }
        }
    }

    // Try package.json
    if let Ok(content) = std::fs::read_to_string(dir.join("package.json")) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(name) = json.get("name").and_then(|n| n.as_str()) {
                return Some(name.to_string());
            }
        }
    }

    // Try go.mod
    if let Ok(content) = std::fs::read_to_string(dir.join("go.mod")) {
        if let Some(line) = content.lines().next() {
            if line.starts_with("module") {
                if let Some(module_path) = line.split_whitespace().nth(1) {
                    // Get the last part of the module path
                    if let Some(name) = module_path.split('/').last() {
                        return Some(name.to_string());
                    }
                }
            }
        }
    }

    // Fallback to directory name
    dir.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_rust_project() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        assert_eq!(detect_project_type(dir.path()), Some(ProjectType::Rust));
    }

    #[test]
    fn test_detect_nodejs_project() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();

        assert_eq!(detect_project_type(dir.path()), Some(ProjectType::NodeJS));
    }

    #[test]
    fn test_detect_go_project() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("go.mod"), "module example.com/test").unwrap();

        assert_eq!(detect_project_type(dir.path()), Some(ProjectType::Go));
    }

    #[test]
    fn test_detect_python_project() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("requirements.txt"), "requests==2.28.0").unwrap();

        assert_eq!(detect_project_type(dir.path()), Some(ProjectType::Python));
    }

    #[test]
    fn test_detect_bun_project() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("bun.lockb"), "").unwrap();

        assert_eq!(detect_project_type(dir.path()), Some(ProjectType::Bun));
    }

    #[test]
    fn test_detect_unknown_project() {
        let dir = tempdir().unwrap();
        assert_eq!(detect_project_type(dir.path()), None);
    }
}
