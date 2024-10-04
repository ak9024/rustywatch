use crate::config::schema::Config;
use std::fs;

pub fn read(path: String) -> Result<Config, Box<dyn std::error::Error>> {
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
    use crate::config::schema::CommandType;

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
 - dir: "/another/directory"
   cmd:
   - "command1"
   - "command2"
   ignore:
   - "file3.txt"
"#;
        let config: Config = serde_yaml::from_str(config_content).unwrap();
        assert_eq!(
            config.workspaces.len(),
            2,
            "Expected exactly two workspaces"
        );

        let workspace1 = &config.workspaces[0];
        assert_eq!(workspace1.dir, "/path/to/directory");
        match &workspace1.cmd {
            CommandType::Single(cmd) => assert_eq!(cmd, "some_command"),
            CommandType::Multiple(_) => panic!("Expected a single command"),
        }
        assert_eq!(
            workspace1.ignore.as_ref().unwrap(),
            &vec!["file1.txt", "file2.txt"]
        );
        assert_eq!(
            workspace1.bin_path.as_ref().unwrap(),
            "/usr/local/bin/executable"
        );
        assert_eq!(
            workspace1.bin_arg.as_ref().unwrap(),
            &vec!["--arg1", "--arg2"]
        );

        let workspace2 = &config.workspaces[1];
        assert_eq!(workspace2.dir, "/another/directory");
        match &workspace2.cmd {
            CommandType::Single(_) => panic!("Expected multiple commands"),
            CommandType::Multiple(cmds) => assert_eq!(cmds, &vec!["command1", "command2"]),
        }
        assert_eq!(workspace2.ignore.as_ref().unwrap(), &vec!["file3.txt"]);
        assert!(workspace2.bin_path.is_none());
        assert!(workspace2.bin_arg.is_none());
    }
}
