use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct Workspace {
    #[validate(length(min = 1))]
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

#[derive(Debug, Deserialize, Validate)]
pub struct Config {
    #[validate(nested)]
    pub workspaces: Vec<Workspace>,
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
