use crate::config::schema::Config;
use std::error::Error;
use std::fs;

const CONFIG_HEADER: &str = "# RustyWatch Configuration
# Documentation: https://github.com/ak9024/rustywatch

";

/// Writes a Config struct to a YAML file with a header comment
pub fn write_config(config: &Config, path: &str) -> Result<(), Box<dyn Error>> {
    let yaml = serde_yaml::to_string(config)?;
    let content = format!("{}{}", CONFIG_HEADER, yaml);
    fs::write(path, content)?;
    Ok(())
}

/// Generates a preview of the config as a formatted YAML string
pub fn preview_config(config: &Config) -> Result<String, Box<dyn Error>> {
    let yaml = serde_yaml::to_string(config)?;
    Ok(format!("{}{}", CONFIG_HEADER, yaml))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::{CommandType, Workspace};
    use tempfile::tempdir;

    #[test]
    fn test_write_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("rustywatch.yaml");

        let config = Config {
            workspaces: vec![Workspace {
                dir: "src".to_string(),
                cmd: CommandType::Single("cargo build".to_string()),
                ignore: Some(vec!["target/".to_string()]),
                bin_path: Some("./target/debug/myapp".to_string()),
                bin_arg: None,
                env_file: None,
            }],
        };

        write_config(&config, config_path.to_str().unwrap()).unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("# RustyWatch Configuration"));
        assert!(content.contains("workspaces:"));
        assert!(content.contains("dir: src"));
        assert!(content.contains("cargo build"));
    }

    #[test]
    fn test_preview_config() {
        let config = Config {
            workspaces: vec![Workspace {
                dir: ".".to_string(),
                cmd: CommandType::Single("npm run dev".to_string()),
                ignore: Some(vec!["node_modules/".to_string()]),
                bin_path: None,
                bin_arg: None,
                env_file: None,
            }],
        };

        let preview = preview_config(&config).unwrap();
        assert!(preview.contains("# RustyWatch Configuration"));
        assert!(preview.contains("npm run dev"));
    }
}
