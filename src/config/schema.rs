use serde::{Deserialize, Serialize};

/// A single project directory that RustyWatch watches and reloads.
///
/// Each workspace runs independently and concurrently with the others defined
/// in a [`Config`]. Commands execute with the workspace [`dir`](Self::dir) as
/// their working directory.
#[derive(Debug, Deserialize, Serialize)]
pub struct Workspace {
    /// Directory to watch for changes; also the working directory for commands.
    pub dir: String,
    /// Command(s) to run when a watched file changes. Accepts either a single
    /// string or a list of strings in YAML (see [`CommandType`]).
    #[serde(deserialize_with = "deserialize_cmd")]
    pub cmd: CommandType,
    /// Glob/name patterns to ignore. When omitted, sensible defaults are used.
    pub ignore: Option<Vec<String>>,
    /// Path to a compiled binary to (re)start after the command succeeds,
    /// relative to [`dir`](Self::dir) unless absolute.
    pub bin_path: Option<String>,
    /// Arguments passed to the binary at [`bin_path`](Self::bin_path).
    pub bin_arg: Option<Vec<String>>,
    /// Optional `.env` file whose variables are injected into the command
    /// environment. Relative to [`dir`](Self::dir) unless it starts with `/`.
    pub env_file: Option<String>,
}

/// One or many shell commands.
///
/// Deserializes transparently from either a YAML scalar (`cmd: "cargo build"`)
/// or a sequence (`cmd: ["npm install", "npm start"]`).
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CommandType {
    /// A single command string.
    Single(String),
    /// Multiple commands executed in parallel.
    Multiple(Vec<String>),
}

/// Top-level configuration parsed from `rustywatch.yaml`.
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// The set of workspaces to watch. Must contain at least one entry.
    pub workspaces: Vec<Workspace>,
}

impl Config {
    /// Validates the configuration.
    ///
    /// Returns an `Err` with a human-readable message if no workspaces are
    /// defined.
    pub fn validate(&self) -> Result<(), String> {
        if self.workspaces.is_empty() {
            return Err("workspaces must be set!".into());
        }

        Ok(())
    }
}

fn deserialize_cmd<'de, D>(deserializer: D) -> Result<CommandType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        String(String),
        Vec(Vec<String>),
    }

    match StringOrVec::deserialize(deserializer)? {
        StringOrVec::String(s) => Ok(CommandType::Single(s)),
        StringOrVec::Vec(v) => Ok(CommandType::Multiple(v)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_workspaces_if_empty() {
        let config = Config { workspaces: vec![] };
        let validate = config.validate();
        match validate {
            Ok(_) => {}
            Err(e) => {
                assert_eq!(e, "workspaces must be set!".to_string())
            }
        }
    }

    #[test]
    fn test_validation_workspaces_not_empty() {
        let config = Config {
            workspaces: vec![Workspace {
                dir: ".".to_string(),
                cmd: CommandType::Single(">".to_string()),
                bin_path: Some(".".to_string()),
                bin_arg: Some(vec![]),
                ignore: Some(vec![]),
                env_file: None,
            }],
        };

        let validate = config.validate();
        assert!(validate.is_ok())
    }

    #[test]
    fn test_workspace_with_single_command() {
        let workspace = Workspace {
            dir: "/path/to/dir".to_string(),
            cmd: CommandType::Single("cargo run".to_string()),
            ignore: None,
            bin_path: None,
            bin_arg: None,
            env_file: None,
        };

        match workspace.cmd {
            CommandType::Single(cmd) => assert_eq!(cmd, "cargo run"),
            CommandType::Multiple(_) => panic!("Expected single command"),
        }
    }

    #[test]
    fn test_workspace_with_multiple_commands() {
        let workspace = Workspace {
            dir: "/path/to/dir".to_string(),
            cmd: CommandType::Multiple(vec!["npm install".to_string(), "npm start".to_string()]),
            ignore: None,
            bin_path: None,
            bin_arg: None,
            env_file: None,
        };

        match workspace.cmd {
            CommandType::Single(_) => panic!("Expected multiple commands"),
            CommandType::Multiple(cmds) => {
                assert_eq!(cmds.len(), 2);
                assert_eq!(cmds[0], "npm install");
                assert_eq!(cmds[1], "npm start");
            }
        }
    }

    #[test]
    fn test_workspace_with_ignore_patterns() {
        let workspace = Workspace {
            dir: ".".to_string(),
            cmd: CommandType::Single("test".to_string()),
            ignore: Some(vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
            ]),
            bin_path: None,
            bin_arg: None,
            env_file: None,
        };

        let ignore = workspace.ignore.unwrap();
        assert_eq!(ignore.len(), 3);
        assert!(ignore.contains(&".git".to_string()));
        assert!(ignore.contains(&"node_modules".to_string()));
        assert!(ignore.contains(&"target".to_string()));
    }

    #[test]
    fn test_workspace_with_bin_args() {
        let workspace = Workspace {
            dir: ".".to_string(),
            cmd: CommandType::Single("cargo build".to_string()),
            ignore: None,
            bin_path: Some("./target/debug/myapp".to_string()),
            bin_arg: Some(vec!["--port".to_string(), "8080".to_string()]),
            env_file: None,
        };

        assert_eq!(workspace.bin_path.unwrap(), "./target/debug/myapp");
        let args = workspace.bin_arg.unwrap();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], "--port");
        assert_eq!(args[1], "8080");
    }

    #[test]
    fn test_command_type_clone() {
        let cmd = CommandType::Single("test".to_string());
        let cloned = cmd.clone();

        match cloned {
            CommandType::Single(c) => assert_eq!(c, "test"),
            _ => panic!("Expected single command"),
        }
    }
}
