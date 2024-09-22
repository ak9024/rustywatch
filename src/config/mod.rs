use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Workspace {
    pub dir: String,
    pub cmd: String,
    pub ignore: Option<Vec<String>>,
    pub bin_path: Option<String>,
    pub bin_arg: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub workspaces: Vec<Workspace>,
}

pub fn read_config(path: String) -> Result<Config, std::io::Error> {
    let config_content = fs::read_to_string(&path).unwrap();
    let config: Config = serde_yaml::from_str(&config_content).unwrap();
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
        for workspace in config.workspaces {
            assert_eq!(workspace.dir, "/path/to/directory");
            assert_eq!(workspace.cmd, "some_command");
            assert_eq!(workspace.ignore.unwrap(), vec!["file1.txt", "file2.txt"]);
            assert_eq!(workspace.bin_path.unwrap(), "/usr/local/bin/executable");
            assert_eq!(workspace.bin_arg.unwrap(), vec!["--arg1", "--arg2"]);
        }
    }
}
