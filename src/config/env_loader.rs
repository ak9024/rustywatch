use log::warn;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Load environment variables from a .env file
/// Returns a HashMap of key-value pairs
pub fn load_env_file(path: &str) -> HashMap<String, String> {
    let mut env_vars = HashMap::new();

    let path = Path::new(path);
    if !path.exists() {
        warn!("Environment file not found: {}", path.display());
        return env_vars;
    }

    match fs::read_to_string(path) {
        Ok(contents) => {
            for line in contents.lines() {
                let line = line.trim();

                // Skip empty lines and comments
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                // Parse KEY=VALUE format
                if let Some((key, value)) = parse_env_line(line) {
                    env_vars.insert(key, value);
                }
            }
        }
        Err(e) => {
            warn!("Failed to read environment file {}: {}", path.display(), e);
        }
    }

    env_vars
}

/// Parse a single line in KEY=VALUE format
fn parse_env_line(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() != 2 {
        return None;
    }

    let key = parts[0].trim().to_string();
    let mut value = parts[1].trim().to_string();

    // Remove surrounding quotes if present
    if (value.starts_with('"') && value.ends_with('"'))
        || (value.starts_with('\'') && value.ends_with('\''))
    {
        value = value[1..value.len() - 1].to_string();
    }

    if key.is_empty() {
        return None;
    }

    Some((key, value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_env_line_simple() {
        let result = parse_env_line("KEY=value");
        assert_eq!(result, Some(("KEY".to_string(), "value".to_string())));
    }

    #[test]
    fn test_parse_env_line_with_quotes() {
        let result = parse_env_line("KEY=\"value with spaces\"");
        assert_eq!(
            result,
            Some(("KEY".to_string(), "value with spaces".to_string()))
        );
    }

    #[test]
    fn test_parse_env_line_with_single_quotes() {
        let result = parse_env_line("KEY='value'");
        assert_eq!(result, Some(("KEY".to_string(), "value".to_string())));
    }

    #[test]
    fn test_parse_env_line_with_equals_in_value() {
        let result = parse_env_line("DATABASE_URL=postgres://user:pass@host/db?option=value");
        assert_eq!(
            result,
            Some((
                "DATABASE_URL".to_string(),
                "postgres://user:pass@host/db?option=value".to_string()
            ))
        );
    }

    #[test]
    fn test_parse_env_line_invalid() {
        assert_eq!(parse_env_line("no_equals_sign"), None);
        assert_eq!(parse_env_line("=no_key"), None);
    }

    #[test]
    fn test_load_env_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "KEY1=value1").unwrap();
        writeln!(temp_file, "KEY2=\"value2\"").unwrap();
        writeln!(temp_file, "KEY3=value with spaces").unwrap();

        let env_vars = load_env_file(temp_file.path().to_str().unwrap());

        assert_eq!(env_vars.get("KEY1"), Some(&"value1".to_string()));
        assert_eq!(env_vars.get("KEY2"), Some(&"value2".to_string()));
        assert_eq!(
            env_vars.get("KEY3"),
            Some(&"value with spaces".to_string())
        );
        assert_eq!(env_vars.len(), 3);
    }

    #[test]
    fn test_load_env_file_not_found() {
        let env_vars = load_env_file("/nonexistent/path/.env");
        assert!(env_vars.is_empty());
    }
}
