use serde::Deserialize;
use std::fs;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct Workspace {
    #[validate(length(min = 1))]
    pub dir: String,
    #[validate(length(min = 1))]
    pub cmd: String,
    pub ignore: Option<Vec<String>>,
    pub bin_path: Option<String>,
    pub bin_arg: Option<Vec<String>>,
}
#[derive(Debug, Deserialize, Validate)]
pub struct Config {
    #[validate(nested)]
    pub workspaces: Vec<Workspace>,
}

pub fn read_config(path: String) -> Result<Config, Box<dyn std::error::Error>> {
    let config_content = fs::read_to_string(&path)?;
    let config: Config = serde_yaml::from_str(&config_content)?;
    if config.workspaces.is_empty() {
        return Err("Config must contain at least one workspace".into());
    }
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_config() {
        let config_content = r#"
workspaces:
 - dir: "/path/to/directory"
   cmd: "some_command"
   ignore:
   - "file1.txt"
   - "file2.txt"
   bin_path: "/usr/local/bin/executable"
   bin_arg:
   - "--arg1"
   - "--arg2"
"#;
        let config: Config = serde_yaml::from_str(config_content).unwrap();
        assert_eq!(config.workspaces.len(), 1, "Expected exactly one workspace");
        let workspace = &config.workspaces[0];
        assert_eq!(workspace.dir, "/path/to/directory");
        assert_eq!(workspace.cmd, "some_command");
        assert_eq!(
            workspace.ignore.as_ref().unwrap(),
            &vec!["file1.txt", "file2.txt"]
        );
        assert_eq!(
            workspace.bin_path.as_ref().unwrap(),
            "/usr/local/bin/executable"
        );
        assert_eq!(
            workspace.bin_arg.as_ref().unwrap(),
            &vec!["--arg1", "--arg2"]
        );
    }
}
