use crate::config::schema::{CommandType, Workspace};

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
    pub default_watch_dir: &'static str,
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
            default_watch_dir: "src",
            default_commands: vec!["cargo build"],
            default_bin_path: Some("./target/debug/{{project_name}}"),
            default_ignore: vec!["target/", ".git/"],
        }
    }

    pub fn nodejs() -> Self {
        Self {
            project_type: ProjectType::NodeJS,
            default_watch_dir: ".",
            default_commands: vec!["npm run dev"],
            default_bin_path: None,
            default_ignore: vec!["node_modules/", ".git/", "dist/", ".next/"],
        }
    }

    pub fn go() -> Self {
        Self {
            project_type: ProjectType::Go,
            default_watch_dir: ".",
            default_commands: vec!["go build"],
            default_bin_path: Some("./{{project_name}}"),
            default_ignore: vec!["vendor/", ".git/"],
        }
    }

    pub fn python() -> Self {
        Self {
            project_type: ProjectType::Python,
            default_watch_dir: ".",
            default_commands: vec!["python main.py"],
            default_bin_path: None,
            default_ignore: vec!["__pycache__/", ".venv/", "venv/", ".git/"],
        }
    }

    pub fn bun() -> Self {
        Self {
            project_type: ProjectType::Bun,
            default_watch_dir: ".",
            default_commands: vec!["bun run start"],
            default_bin_path: None,
            default_ignore: vec!["node_modules/", ".git/"],
        }
    }

    pub fn other() -> Self {
        Self {
            project_type: ProjectType::Other,
            default_watch_dir: ".",
            default_commands: vec!["echo 'Add your command here'"],
            default_bin_path: None,
            default_ignore: vec![".git/"],
        }
    }

    /// Convert template to a Workspace with optional project name substitution
    pub fn to_workspace(&self, project_name: Option<&str>) -> Workspace {
        let cmd = self.default_commands.join("; ");

        let bin_path = self.default_bin_path.map(|path| {
            if let Some(name) = project_name {
                path.replace("{{project_name}}", name)
            } else {
                path.replace("{{project_name}}", "app")
            }
        });

        Workspace {
            dir: self.default_watch_dir.to_string(),
            cmd: CommandType::Single(cmd),
            ignore: Some(self.default_ignore.iter().map(|s| s.to_string()).collect()),
            bin_path,
            bin_arg: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_template() {
        let template = ProjectTemplate::rust();
        assert_eq!(template.project_type, ProjectType::Rust);
        assert_eq!(template.default_watch_dir, "src");
        assert!(template.default_bin_path.is_some());
    }

    #[test]
    fn test_nodejs_template() {
        let template = ProjectTemplate::nodejs();
        assert_eq!(template.project_type, ProjectType::NodeJS);
        assert!(template.default_bin_path.is_none());
    }

    #[test]
    fn test_to_workspace_with_project_name() {
        let template = ProjectTemplate::rust();
        let workspace = template.to_workspace(Some("myapp"));

        assert_eq!(workspace.dir, "src");
        assert_eq!(workspace.bin_path, Some("./target/debug/myapp".to_string()));
    }

    #[test]
    fn test_to_workspace_without_project_name() {
        let template = ProjectTemplate::rust();
        let workspace = template.to_workspace(None);

        assert_eq!(workspace.bin_path, Some("./target/debug/app".to_string()));
    }

    #[test]
    fn test_project_type_display() {
        assert_eq!(ProjectType::Rust.display_name(), "Rust");
        assert_eq!(ProjectType::NodeJS.display_name(), "Node.js");
        assert_eq!(ProjectType::Go.display_name(), "Go");
    }
}
