use super::templates::ProjectType;
use std::fmt;
use std::path::{Path, PathBuf};

/// Directory names never descended into while scanning for projects.
pub const SKIP_DIRS: &[&str] = &[
    "target",
    "node_modules",
    "vendor",
    "dist",
    "build",
    "out",
    "coverage",
    "__pycache__",
    "venv",
    ".venv",
    ".git",
];

/// A project found on disk by [`scan_projects`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedProject {
    /// Directory holding the marker file, relative to the scan root.
    pub dir: PathBuf,
    /// Which language/toolchain the marker file implies.
    pub project_type: ProjectType,
    /// Project name read from the manifest, or the directory name.
    pub name: Option<String>,
}

impl fmt::Display for DetectedProject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dir = self.dir.to_string_lossy();
        let dir = if dir.is_empty() { "." } else { dir.as_ref() };

        match &self.name {
            Some(name) => write!(f, "{dir}  ({} — {name})", self.project_type),
            None => write!(f, "{dir}  ({})", self.project_type),
        }
    }
}

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

/// Walks `root` and up to `max_depth` levels of subdirectories, returning every
/// project found.
///
/// The root is reported first when it holds a marker file; the rest are sorted
/// by path. Build output and dependency directories ([`SKIP_DIRS`]) and hidden
/// directories are never descended into, so a scan of a monorepo returns its
/// actual projects rather than every vendored copy of them.
pub fn scan_projects(root: &Path, max_depth: usize) -> Vec<DetectedProject> {
    let mut found = Vec::new();
    collect_projects(root, root, 0, max_depth, &mut found);

    // Root first, then by path, so the list reads the way the tree does.
    found.sort_by(|a, b| {
        let a_is_root = a.dir.as_os_str().is_empty();
        let b_is_root = b.dir.as_os_str().is_empty();
        b_is_root.cmp(&a_is_root).then_with(|| a.dir.cmp(&b.dir))
    });

    found
}

fn collect_projects(
    root: &Path,
    dir: &Path,
    depth: usize,
    max_depth: usize,
    found: &mut Vec<DetectedProject>,
) {
    if let Some(project_type) = detect_project_type(dir) {
        found.push(DetectedProject {
            dir: dir.strip_prefix(root).unwrap_or(dir).to_path_buf(),
            project_type,
            name: detect_project_name(dir),
        });
    }

    if depth >= max_depth {
        return;
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    let mut children: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && !is_skipped(path))
        .collect();
    children.sort();

    for child in children {
        collect_projects(root, &child, depth + 1, max_depth, found);
    }
}

fn is_skipped(path: &Path) -> bool {
    match path.file_name().and_then(|name| name.to_str()) {
        Some(name) => name.starts_with('.') || SKIP_DIRS.contains(&name),
        None => true,
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
                    if let Some(name) = module_path.split('/').next_back() {
                        return Some(name.to_string());
                    }
                }
            }
        }
    }

    // Fallback to directory name
    dir.canonicalize()
        .ok()
        .as_deref()
        .unwrap_or(dir)
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
}

/// Picks the script to run for a JavaScript project, in `package.json` order of
/// preference, falling back to the template default when nothing matches.
///
/// `runner` is the package manager binary (`npm`, `bun`, …).
pub fn detect_js_command(dir: &Path, runner: &str) -> Option<String> {
    let content = std::fs::read_to_string(dir.join("package.json")).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let scripts = json.get("scripts")?.as_object()?;

    ["dev", "start", "serve", "watch"]
        .iter()
        .find(|script| scripts.contains_key(**script))
        .map(|script| format!("{runner} run {script}"))
}

/// Picks the entry point for a Python project, so the generated command points
/// at a file that actually exists.
pub fn detect_python_entry(dir: &Path) -> Option<String> {
    ["main.py", "app.py", "manage.py", "run.py", "__main__.py"]
        .iter()
        .find(|candidate| dir.join(candidate).exists())
        .map(|candidate| (*candidate).to_string())
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

    #[test]
    fn test_scan_finds_nested_projects() {
        let root = tempdir().unwrap();
        fs::create_dir_all(root.path().join("services/api")).unwrap();
        fs::create_dir_all(root.path().join("web")).unwrap();

        fs::write(
            root.path().join("services/api/go.mod"),
            "module example.com/api",
        )
        .unwrap();
        fs::write(root.path().join("web/package.json"), r#"{"name":"web"}"#).unwrap();

        let found = scan_projects(root.path(), 2);
        let dirs: Vec<_> = found
            .iter()
            .map(|p| p.dir.to_string_lossy().to_string())
            .collect();

        assert_eq!(dirs, vec!["services/api".to_string(), "web".to_string()]);
        assert_eq!(found[0].project_type, ProjectType::Go);
        assert_eq!(found[0].name.as_deref(), Some("api"));
        assert_eq!(found[1].project_type, ProjectType::NodeJS);
    }

    #[test]
    fn test_scan_reports_root_first() {
        let root = tempdir().unwrap();
        fs::create_dir_all(root.path().join("api")).unwrap();
        fs::write(root.path().join("package.json"), r#"{"name":"root"}"#).unwrap();
        fs::write(root.path().join("api/go.mod"), "module example.com/api").unwrap();

        let found = scan_projects(root.path(), 2);

        assert_eq!(found.len(), 2);
        assert_eq!(found[0].dir, PathBuf::from(""));
        assert_eq!(found[0].project_type, ProjectType::NodeJS);
        assert_eq!(found[1].dir, PathBuf::from("api"));
    }

    #[test]
    fn test_scan_skips_build_and_hidden_directories() {
        let root = tempdir().unwrap();
        for skipped in ["node_modules/dep", "target/debug", ".git/objects"] {
            fs::create_dir_all(root.path().join(skipped)).unwrap();
            fs::write(root.path().join(skipped).join("package.json"), "{}").unwrap();
        }

        assert!(scan_projects(root.path(), 3).is_empty());
    }

    #[test]
    fn test_scan_respects_depth() {
        let root = tempdir().unwrap();
        fs::create_dir_all(root.path().join("a/b")).unwrap();
        fs::write(root.path().join("a/b/go.mod"), "module example.com/deep").unwrap();

        assert!(scan_projects(root.path(), 1).is_empty());
        assert_eq!(scan_projects(root.path(), 2).len(), 1);
    }

    #[test]
    fn test_scan_empty_directory() {
        let root = tempdir().unwrap();
        assert!(scan_projects(root.path(), 2).is_empty());
    }

    #[test]
    fn test_detect_js_command_prefers_dev() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"scripts":{"start":"node .","dev":"vite"}}"#,
        )
        .unwrap();

        assert_eq!(
            detect_js_command(dir.path(), "npm").as_deref(),
            Some("npm run dev")
        );
    }

    #[test]
    fn test_detect_js_command_falls_back_to_start() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"scripts":{"start":"bun index.ts"}}"#,
        )
        .unwrap();

        assert_eq!(
            detect_js_command(dir.path(), "bun").as_deref(),
            Some("bun run start")
        );
    }

    #[test]
    fn test_detect_js_command_without_scripts() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("package.json"), r#"{"name":"x"}"#).unwrap();

        assert!(detect_js_command(dir.path(), "npm").is_none());
    }

    #[test]
    fn test_detect_python_entry() {
        let dir = tempdir().unwrap();
        assert!(detect_python_entry(dir.path()).is_none());

        fs::write(dir.path().join("app.py"), "").unwrap();
        assert_eq!(detect_python_entry(dir.path()).as_deref(), Some("app.py"));
    }

    #[test]
    fn test_detect_project_name_falls_back_to_directory() {
        let root = tempdir().unwrap();
        let dir = root.path().join("my-service");
        fs::create_dir_all(&dir).unwrap();

        assert_eq!(detect_project_name(&dir).as_deref(), Some("my-service"));
    }
}
