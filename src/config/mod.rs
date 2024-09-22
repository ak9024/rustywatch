use std::fs;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub dir: String,
    pub cmd: String,
    pub ignore: Option<Vec<String>>,
    pub bin_path: Option<String>,
    pub bin_arg: Option<Vec<String>>,
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
dir: "/path/to/directory"
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

        assert_eq!(config.dir, "/path/to/directory");
        assert_eq!(config.cmd, "some_command");
        assert_eq!(config.ignore.unwrap(), vec!["file1.txt", "file2.txt"]);
        assert_eq!(config.bin_path.unwrap(), "/usr/local/bin/executable");
        assert_eq!(config.bin_arg.unwrap(), vec!["--arg1", "--arg2"]);
    }
}
