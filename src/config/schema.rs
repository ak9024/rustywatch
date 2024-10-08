use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Workspace {
    pub dir: String,
    #[serde(deserialize_with = "deserialize_cmd")]
    pub cmd: CommandType,
    pub ignore: Option<Vec<String>>,
    pub bin_path: Option<String>,
    pub bin_arg: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum CommandType {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Deserialize)]
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
}
