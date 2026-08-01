use super::detect;
use crate::config::schema::Workspace;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    Rust,
    NodeJS,
    Go,
    Python,
    Bun,
    Other,
}

impl ProjectType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ProjectType::Rust => "Rust",
            ProjectType::NodeJS => "Node.js",
            ProjectType::Go => "Go",
            ProjectType::Python => "Python",
            ProjectType::Bun => "Bun",
            ProjectType::Other => "Other",
        }
    }

    pub fn all() -> Vec<ProjectType> {
        vec![
            ProjectType::Rust,
            ProjectType::NodeJS,
            ProjectType::Go,
            ProjectType::Python,
            ProjectType::Bun,
            ProjectType::Other,
        ]
    }
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

pub struct ProjectTemplate {
    pub project_type: ProjectType,
    pub default_commands: Vec<&'static str>,
    pub default_bin_path: Option<&'static str>,
    pub default_ignore: Vec<&'static str>,
}

impl ProjectTemplate {
    pub fn for_type(project_type: ProjectType) -> Self {
        match project_type {
            ProjectType::Rust => Self::rust(),
            ProjectType::NodeJS => Self::nodejs(),
            ProjectType::Go => Self::go(),
            ProjectType::Python => Self::python(),
            ProjectType::Bun => Self::bun(),
            ProjectType::Other => Self::other(),
        }
    }

    pub fn rust() -> Self {
        Self {
            project_type: ProjectType::Rust,
            default_commands: vec!["cargo build"],
            // Relative to the workspace `dir`, which is the crate root — not
            // `src`, or the binary would be looked for under `src/target/`.
            default_bin_path: Some("./target/debug/{{project_name}}"),
            default_ignore: vec!["target/", ".git/"],
        }
    }

    pub fn nodejs() -> Self {
        Self {
            project_type: ProjectType::NodeJS,
            default_commands: vec!["npm run dev"],
            default_bin_path: None,
            default_ignore: vec!["node_modules/", ".git/", "dist/", ".next/"],
        }
    }

    pub fn go() -> Self {
        Self {
            project_type: ProjectType::Go,
            default_commands: vec!["go build"],
            default_bin_path: Some("./{{project_name}}"),
            default_ignore: vec!["vendor/", ".git/"],
        }
    }

    pub fn python() -> Self {
        Self {
            project_type: ProjectType::Python,
            default_commands: vec!["python main.py"],
            default_bin_path: None,
            default_ignore: vec!["__pycache__/", ".venv/", "venv/", ".git/"],
        }
    }

    pub fn bun() -> Self {
        Self {
            project_type: ProjectType::Bun,
            default_commands: vec!["bun run start"],
            default_bin_path: None,
            default_ignore: vec!["node_modules/", ".git/"],
        }
    }

    pub fn other() -> Self {
        Self {
            project_type: ProjectType::Other,
            default_commands: vec!["echo 'Add your command here'"],
            default_bin_path: None,
            default_ignore: vec![".git/"],
        }
    }

    /// The command to run for a project living at `fs_dir`.
    ///
    /// Manifests are probed so the generated command points at something that
    /// exists: a JavaScript project gets whichever of `dev`/`start`/`serve`/
    /// `watch` its `package.json` actually declares, and a Python project gets
    /// whichever entry file it actually has. Falls back to the static template
    /// default when nothing can be read.
    pub fn command_for(&self, fs_dir: &Path) -> String {
        let detected = match self.project_type {
            ProjectType::NodeJS => detect::detect_js_command(fs_dir, "npm"),
            ProjectType::Bun => detect::detect_js_command(fs_dir, "bun"),
            ProjectType::Python => {
                detect::detect_python_entry(fs_dir).map(|entry| format!("python {entry}"))
            }
            _ => None,
        };

        detected.unwrap_or_else(|| self.default_commands.join(" && "))
    }

    /// The binary path for this template, with `{{project_name}}` substituted.
    ///
    /// The result is relative to the workspace `dir`, matching how RustyWatch
    /// resolves `bin_path` at run time.
    pub fn bin_path_for(&self, project_name: Option<&str>) -> Option<String> {
        self.default_bin_path
            .map(|path| path.replace("{{project_name}}", project_name.unwrap_or("app")))
    }

    /// The ignore patterns for this template.
    pub fn ignore_patterns(&self) -> Vec<String> {
        self.default_ignore.iter().map(|s| s.to_string()).collect()
    }

    /// Builds a ready-to-write [`Workspace`].
    ///
    /// `config_dir` is the path written into the config (relative to the config
    /// file); `fs_dir` is where that project lives right now, used only for
    /// probing manifests.
    pub fn to_workspace(
        &self,
        config_dir: &str,
        fs_dir: &Path,
        project_name: Option<&str>,
    ) -> Workspace {
        let mut workspace = Workspace::new(config_dir)
            .cmd(self.command_for(fs_dir))
            .ignore(self.ignore_patterns());

        if let Some(bin_path) = self.bin_path_for(project_name) {
            workspace = workspace.bin_path(bin_path);
        }

        // Only offer an env file the project actually has.
        if fs_dir.join(".env").exists() {
            workspace = workspace.env_file(".env");
        }

        workspace
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_rust_template() {
        let template = ProjectTemplate::rust();
        assert_eq!(template.project_type, ProjectType::Rust);
        assert!(template.default_bin_path.is_some());
    }

    #[test]
    fn test_nodejs_template() {
        let template = ProjectTemplate::nodejs();
        assert_eq!(template.project_type, ProjectType::NodeJS);
        assert!(template.default_bin_path.is_none());
    }

    #[test]
    fn test_bin_path_for_with_project_name() {
        assert_eq!(
            ProjectTemplate::rust().bin_path_for(Some("myapp")),
            Some("./target/debug/myapp".to_string())
        );
    }

    #[test]
    fn test_bin_path_for_without_project_name() {
        assert_eq!(
            ProjectTemplate::rust().bin_path_for(None),
            Some("./target/debug/app".to_string())
        );
    }

    // A Rust workspace must watch the crate root: with `dir: src` the relative
    // `bin_path` would resolve to `src/target/debug/<name>`, which never exists.
    #[test]
    fn test_rust_workspace_watches_crate_root() {
        let dir = tempdir().unwrap();
        let workspace = ProjectTemplate::rust().to_workspace(".", dir.path(), Some("demo"));

        assert_eq!(workspace.dir, ".");
        assert_eq!(workspace.bin_path.as_deref(), Some("./target/debug/demo"));
    }

    #[test]
    fn test_to_workspace_uses_given_config_dir() {
        let dir = tempdir().unwrap();
        let workspace = ProjectTemplate::go().to_workspace("services/api", dir.path(), Some("api"));

        assert_eq!(workspace.dir, "services/api");
        assert_eq!(workspace.bin_path.as_deref(), Some("./api"));
    }

    #[test]
    fn test_command_for_reads_package_json_scripts() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{"scripts":{"serve":"vite preview"}}"#,
        )
        .unwrap();

        assert_eq!(
            ProjectTemplate::nodejs().command_for(dir.path()),
            "npm run serve"
        );
    }

    #[test]
    fn test_command_for_falls_back_to_template_default() {
        let dir = tempdir().unwrap();
        assert_eq!(
            ProjectTemplate::nodejs().command_for(dir.path()),
            "npm run dev"
        );
    }

    #[test]
    fn test_command_for_python_uses_existing_entry() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("manage.py"), "").unwrap();

        assert_eq!(
            ProjectTemplate::python().command_for(dir.path()),
            "python manage.py"
        );
    }

    #[test]
    fn test_to_workspace_picks_up_env_file() {
        let dir = tempdir().unwrap();
        assert!(ProjectTemplate::go()
            .to_workspace(".", dir.path(), None)
            .env_file
            .is_none());

        fs::write(dir.path().join(".env"), "PORT=8080").unwrap();
        assert_eq!(
            ProjectTemplate::go()
                .to_workspace(".", dir.path(), None)
                .env_file
                .as_deref(),
            Some(".env")
        );
    }

    #[test]
    fn test_generated_workspaces_validate() {
        let dir = tempdir().unwrap();

        for project_type in ProjectType::all() {
            let workspace =
                ProjectTemplate::for_type(project_type).to_workspace(".", dir.path(), Some("demo"));

            workspace
                .validate()
                .unwrap_or_else(|e| panic!("{project_type} template is invalid: {e}"));
        }
    }

    #[test]
    fn test_project_type_display() {
        assert_eq!(ProjectType::Rust.display_name(), "Rust");
        assert_eq!(ProjectType::NodeJS.display_name(), "Node.js");
        assert_eq!(ProjectType::Go.display_name(), "Go");
        assert_eq!(ProjectType::Bun.to_string(), "Bun");
    }
}
