use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Workspace {
    pub dir: String,
    #[serde(deserialize_with = "deserialize_cmd")]
    pub cmd: CommandType,
    pub ignore: Option<Vec<String>>,
    pub bin_path: Option<String>,
    pub bin_arg: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CommandType {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub workspaces: Vec<Workspace>,
}

impl Config {
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
            ignore: Some(vec![".git".to_string(), "node_modules".to_string(), "target".to_string()]),
            bin_path: None,
            bin_arg: None,
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
