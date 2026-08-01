use crate::config::schema::Config;
use std::error::Error;
use std::fs;

const CONFIG_HEADER: &str = "# RustyWatch Configuration
# Documentation: https://rustywatch.vercel.app/reference/reference/
#
# `cmd` runs with `dir` as its working directory - no `cd` prefix needed.
# `bin_path` is resolved relative to `dir` unless it is absolute.
# A non-empty `ignore` list replaces the built-in defaults rather than adding
# to them.

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
    use crate::config::schema::Workspace;
    use tempfile::tempdir;

    #[test]
    fn test_write_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("rustywatch.yaml");

        let config = Config::new([Workspace::new(".")
            .cmd("cargo build")
            .ignore(["target/"])
            .bin_path("./target/debug/myapp")]);

        write_config(&config, config_path.to_str().unwrap()).unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("# RustyWatch Configuration"));
        assert!(content.contains("workspaces:"));
        assert!(content.contains("dir: ."));
        assert!(content.contains("cargo build"));
    }

    #[test]
    fn test_preview_config() {
        let config = Config::new([Workspace::new(".")
            .cmd("npm run dev")
            .ignore(["node_modules/"])]);

        let preview = preview_config(&config).unwrap();
        assert!(preview.contains("# RustyWatch Configuration"));
        assert!(preview.contains("npm run dev"));
    }

    // Unset optional fields must be skipped, not emitted as `null` - a written
    // `bin_path: null` would round-trip back as an unusable value.
    #[test]
    fn test_written_config_has_no_nulls_and_reloads() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("rustywatch.yaml");

        let config = Config::new([Workspace::new(".").cmd("cargo build")]);
        write_config(&config, config_path.to_str().unwrap()).unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(!content.contains("null"));
        assert_eq!(Config::from_file(&config_path).unwrap(), config);
    }
}
