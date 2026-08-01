use crate::config::env_loader::load_env_file;
use crate::error::{Error, Result};
use crate::watch::ignore_defaults::{default_ignore_patterns, merge_with_defaults};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A single project directory that RustyWatch watches and reloads.
///
/// Each workspace runs independently and concurrently with the others defined
/// in a [`Config`]. Commands execute with the workspace [`dir`](Self::dir) as
/// their working directory.
///
/// Build one with [`Workspace::new`] and the chainable setters rather than a
/// struct literal — the fields stay public, but the builder keeps call sites
/// readable and survives new optional fields being added:
///
/// ```
/// use rustywatch::Workspace;
///
/// let workspace = Workspace::new("./api")
///     .cmd("cargo build")
///     .bin_path("target/debug/api")
///     .bin_arg(["--port", "8080"])
///     .extend_ignore(["*.snap"])
///     .env_file(".env");
///
/// assert_eq!(workspace.dir, "./api");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Workspace {
    /// Directory to watch for changes; also the working directory for commands.
    pub dir: String,
    /// Command(s) to run when a watched file changes. Accepts either a single
    /// string or a list of strings in YAML (see [`CommandType`]).
    #[serde(deserialize_with = "deserialize_cmd")]
    pub cmd: CommandType,
    /// Glob/name patterns to ignore. When omitted, sensible defaults are used.
    ///
    /// A non-empty list *replaces* [`DEFAULT_IGNORE_PATTERNS`] rather than
    /// extending them; use [`Workspace::extend_ignore`] to keep the defaults.
    ///
    /// [`DEFAULT_IGNORE_PATTERNS`]: crate::watch::ignore_defaults::DEFAULT_IGNORE_PATTERNS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore: Option<Vec<String>>,
    /// Path to a compiled binary to (re)start after the command succeeds,
    /// relative to [`dir`](Self::dir) unless absolute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin_path: Option<String>,
    /// Arguments passed to the binary at [`bin_path`](Self::bin_path).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin_arg: Option<Vec<String>>,
    /// Optional `.env` file whose variables are injected into the command
    /// environment. Relative to [`dir`](Self::dir) unless it starts with `/`,
    /// which means "relative to the project root".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_file: Option<String>,
}

impl Workspace {
    /// Starts a workspace watching `dir`, with no command yet.
    ///
    /// `dir` is both the watched directory and the working directory for every
    /// command and binary spawn, so commands must not be prefixed with `cd`.
    pub fn new(dir: impl Into<String>) -> Self {
        Self {
            dir: dir.into(),
            cmd: CommandType::Single(String::new()),
            ignore: None,
            bin_path: None,
            bin_arg: None,
            env_file: None,
        }
    }

    /// Sets a single command to run on change.
    pub fn cmd(mut self, cmd: impl Into<String>) -> Self {
        self.cmd = CommandType::Single(cmd.into());
        self
    }

    /// Sets several commands to run on change.
    ///
    /// They execute **in parallel**, not in sequence — see [`CommandType`].
    pub fn cmds<I, S>(mut self, cmds: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.cmd = CommandType::Multiple(cmds.into_iter().map(Into::into).collect());
        self
    }

    /// Sets the compiled binary to restart after the command(s) finish.
    ///
    /// Relative paths resolve against [`dir`](Self::dir), not the process
    /// working directory.
    pub fn bin_path(mut self, bin_path: impl Into<String>) -> Self {
        self.bin_path = Some(bin_path.into());
        self
    }

    /// Sets the arguments passed to the binary at [`bin_path`](Self::bin_path).
    pub fn bin_arg<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.bin_arg = Some(args.into_iter().map(Into::into).collect());
        self
    }

    /// Replaces the default ignore patterns with `patterns`.
    ///
    /// This drops `target/`, `node_modules/` and every other default. Use
    /// [`extend_ignore`](Self::extend_ignore) to keep them.
    pub fn ignore<I, S>(mut self, patterns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.ignore = Some(patterns.into_iter().map(Into::into).collect());
        self
    }

    /// Adds `patterns` on top of whatever is already ignored.
    ///
    /// When no patterns have been set yet this seeds the list with
    /// [`crate::default_ignore_patterns`] first, so the defaults survive.
    pub fn extend_ignore<I, S>(mut self, patterns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut current = match self.ignore.take() {
            Some(existing) if !existing.is_empty() => existing,
            _ => default_ignore_patterns(),
        };
        current.extend(patterns.into_iter().map(Into::into));
        self.ignore = Some(current);
        self
    }

    /// Sets the `.env` file to load before running commands.
    ///
    /// A leading `/` means "relative to the project root" — it is *not* a
    /// filesystem-absolute path. Anything else is relative to
    /// [`dir`](Self::dir).
    pub fn env_file(mut self, env_file: impl Into<String>) -> Self {
        self.env_file = Some(env_file.into());
        self
    }

    /// Returns the ignore patterns actually used, with defaults applied.
    pub fn ignore_patterns(&self) -> Vec<String> {
        merge_with_defaults(self.ignore.clone())
    }

    /// Resolves [`env_file`](Self::env_file) into concrete environment
    /// variables.
    ///
    /// Returns an empty map when no file is configured, or when the file is
    /// missing or unreadable (a warning is logged in that case).
    pub fn env_vars(&self) -> HashMap<String, String> {
        match &self.env_file {
            Some(env_file) => {
                let resolved = match env_file.strip_prefix('/') {
                    Some(from_root) => from_root.to_string(),
                    None => format!("{}/{}", self.dir, env_file),
                };
                load_env_file(&resolved)
            }
            None => HashMap::new(),
        }
    }

    /// Validates this workspace in isolation.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidConfig`] if `dir` is empty or no command is set,
    /// and [`Error::Ignore`] if an ignore pattern is not a valid glob.
    pub fn validate(&self) -> Result<()> {
        if self.dir.trim().is_empty() {
            return Err(Error::invalid_config("workspace `dir` must not be empty"));
        }

        if self.cmd.is_empty() {
            return Err(Error::invalid_config(format!(
                "workspace `{}` has no command to run",
                self.dir
            )));
        }

        // Compile eagerly so a bad glob fails at build time instead of silently
        // ignoring nothing once the watcher is already running.
        crate::watch::filter::CompiledFilter::new(&self.ignore_patterns())?;

        Ok(())
    }
}

/// One or many shell commands.
///
/// Deserializes transparently from either a YAML scalar (`cmd: "cargo build"`)
/// or a sequence (`cmd: ["npm install", "npm start"]`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CommandType {
    /// A single command string.
    Single(String),
    /// Multiple commands executed **in parallel**, not in sequence. The list
    /// syntax reads like ordered steps but is not ordered — chain with `&&`
    /// inside a single command when you need sequencing.
    Multiple(Vec<String>),
}

impl CommandType {
    /// Returns the commands as a slice-like iterator, in declaration order.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        match self {
            CommandType::Single(cmd) => std::slice::from_ref(cmd),
            CommandType::Multiple(cmds) => cmds.as_slice(),
        }
        .iter()
        .map(String::as_str)
    }

    /// Returns `true` when there is nothing to execute.
    pub fn is_empty(&self) -> bool {
        self.iter().all(|cmd| cmd.trim().is_empty())
    }

    /// Number of commands held.
    pub fn len(&self) -> usize {
        match self {
            CommandType::Single(_) => 1,
            CommandType::Multiple(cmds) => cmds.len(),
        }
    }
}

impl From<&str> for CommandType {
    fn from(cmd: &str) -> Self {
        CommandType::Single(cmd.to_string())
    }
}

impl From<String> for CommandType {
    fn from(cmd: String) -> Self {
        CommandType::Single(cmd)
    }
}

impl From<Vec<String>> for CommandType {
    fn from(cmds: Vec<String>) -> Self {
        CommandType::Multiple(cmds)
    }
}

/// Top-level configuration parsed from `rustywatch.yaml`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Config {
    /// The set of workspaces to watch. Must contain at least one entry.
    pub workspaces: Vec<Workspace>,
}

impl Config {
    /// Builds a config from any iterator of workspaces.
    pub fn new<I: IntoIterator<Item = Workspace>>(workspaces: I) -> Self {
        Self {
            workspaces: workspaces.into_iter().collect(),
        }
    }

    /// Reads and deserializes a YAML configuration file.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ConfigRead`] if the file cannot be read and
    /// [`Error::ConfigParse`] if its contents are not valid RustyWatch YAML.
    /// The config is *not* validated — call [`validate`](Self::validate) or go
    /// through [`Watcher::from_config_file`](crate::Watcher::from_config_file).
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let display = path.display().to_string();

        let contents = fs::read_to_string(path).map_err(|source| Error::ConfigRead {
            path: display.clone(),
            source,
        })?;

        serde_yaml::from_str(&contents).map_err(|source| Error::ConfigParse {
            path: display,
            source,
        })
    }

    /// Parses a config from a YAML string.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ConfigParse`] if the YAML is not a valid config.
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml).map_err(|source| Error::ConfigParse {
            path: "<string>".to_string(),
            source,
        })
    }

    /// Serializes the config back to YAML.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ConfigParse`] if serialization fails.
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).map_err(|source| Error::ConfigParse {
            path: "<string>".to_string(),
            source,
        })
    }

    /// Validates the configuration and every workspace it contains.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidConfig`] if no workspaces are defined, or the
    /// first per-workspace failure from [`Workspace::validate`].
    pub fn validate(&self) -> Result<()> {
        if self.workspaces.is_empty() {
            return Err(Error::invalid_config("`workspaces` must not be empty"));
        }

        for workspace in &self.workspaces {
            workspace.validate()?;
        }

        Ok(())
    }
}

fn deserialize_cmd<'de, D>(deserializer: D) -> std::result::Result<CommandType, D::Error>
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
        let err = config.validate().unwrap_err();

        assert!(matches!(err, Error::InvalidConfig(_)));
        assert!(err.to_string().contains("`workspaces` must not be empty"));
    }

    #[test]
    fn test_validation_workspaces_not_empty() {
        let config = Config::new([Workspace::new(".").cmd("echo hi")]);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validation_rejects_empty_dir() {
        let config = Config::new([Workspace::new("  ").cmd("echo hi")]);
        let err = config.validate().unwrap_err();

        assert!(err.to_string().contains("`dir` must not be empty"));
    }

    #[test]
    fn test_validation_rejects_empty_command() {
        let config = Config::new([Workspace::new("./api")]);
        let err = config.validate().unwrap_err();

        assert!(err.to_string().contains("has no command to run"));
    }

    #[test]
    fn test_validation_rejects_bad_glob() {
        let config = Config::new([Workspace::new(".").cmd("echo hi").ignore(["src/**/["])]);
        let err = config.validate().unwrap_err();

        assert!(matches!(err, Error::Ignore { .. }));
    }

    #[test]
    fn test_builder_sets_every_field() {
        let workspace = Workspace::new("./api")
            .cmd("cargo build")
            .bin_path("target/debug/api")
            .bin_arg(["--port", "8080"])
            .ignore(["target/"])
            .env_file(".env");

        assert_eq!(workspace.dir, "./api");
        assert_eq!(
            workspace.cmd,
            CommandType::Single("cargo build".to_string())
        );
        assert_eq!(workspace.bin_path.as_deref(), Some("target/debug/api"));
        assert_eq!(
            workspace.bin_arg.as_deref(),
            Some(["--port".to_string(), "8080".to_string()].as_slice())
        );
        assert_eq!(
            workspace.ignore.as_deref(),
            Some(["target/".to_string()].as_slice())
        );
        assert_eq!(workspace.env_file.as_deref(), Some(".env"));
    }

    #[test]
    fn test_builder_cmds_is_multiple() {
        let workspace = Workspace::new(".").cmds(["npm install", "npm start"]);

        assert_eq!(
            workspace.cmd,
            CommandType::Multiple(vec!["npm install".to_string(), "npm start".to_string()])
        );
    }

    #[test]
    fn test_ignore_replaces_defaults() {
        let workspace = Workspace::new(".").cmd("echo").ignore([".git"]);
        let patterns = workspace.ignore_patterns();

        assert_eq!(patterns, vec![".git".to_string()]);
        assert!(!patterns.contains(&"target/".to_string()));
    }

    #[test]
    fn test_extend_ignore_keeps_defaults() {
        let workspace = Workspace::new(".").cmd("echo").extend_ignore(["*.snap"]);
        let patterns = workspace.ignore_patterns();

        assert!(patterns.contains(&"target/".to_string()));
        assert!(patterns.contains(&"node_modules/".to_string()));
        assert!(patterns.contains(&"*.snap".to_string()));
    }

    #[test]
    fn test_extend_ignore_appends_to_explicit_list() {
        let workspace = Workspace::new(".")
            .cmd("echo")
            .ignore([".git"])
            .extend_ignore(["*.snap"]);

        assert_eq!(
            workspace.ignore.unwrap(),
            vec![".git".to_string(), "*.snap".to_string()]
        );
    }

    #[test]
    fn test_no_ignore_uses_defaults() {
        let patterns = Workspace::new(".").cmd("echo").ignore_patterns();
        assert!(patterns.contains(&".git/".to_string()));
    }

    #[test]
    fn test_command_type_iter_and_len() {
        let single = CommandType::Single("cargo build".to_string());
        assert_eq!(single.len(), 1);
        assert_eq!(single.iter().collect::<Vec<_>>(), vec!["cargo build"]);
        assert!(!single.is_empty());

        let multiple = CommandType::from(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(multiple.len(), 2);
        assert_eq!(multiple.iter().collect::<Vec<_>>(), vec!["a", "b"]);
    }

    #[test]
    fn test_command_type_is_empty() {
        assert!(CommandType::Single(String::new()).is_empty());
        assert!(CommandType::Single("   ".to_string()).is_empty());
        assert!(CommandType::Multiple(vec![]).is_empty());
        assert!(!CommandType::from("echo").is_empty());
    }

    #[test]
    fn test_env_vars_without_env_file() {
        assert!(Workspace::new(".").cmd("echo").env_vars().is_empty());
    }

    #[test]
    fn test_env_vars_from_workspace_relative_file() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let mut file = fs::File::create(dir.path().join(".env")).unwrap();
        writeln!(file, "GREETING=hello").unwrap();

        let workspace = Workspace::new(dir.path().to_str().unwrap())
            .cmd("echo")
            .env_file(".env");

        assert_eq!(
            workspace.env_vars().get("GREETING"),
            Some(&"hello".to_string())
        );
    }

    #[test]
    fn test_config_yaml_round_trip() {
        let config = Config::new([Workspace::new("./api").cmd("cargo build")]);
        let yaml = config.to_yaml().unwrap();

        // Unset optional fields are skipped rather than emitted as `null`.
        assert!(!yaml.contains("null"));
        assert_eq!(Config::from_yaml(&yaml).unwrap(), config);
    }

    #[test]
    fn test_config_from_yaml_rejects_garbage() {
        let err = Config::from_yaml("workspaces: [").unwrap_err();
        assert!(matches!(err, Error::ConfigParse { .. }));
    }

    #[test]
    fn test_config_from_yaml_full_document() {
        let config = Config::from_yaml(
            r#"
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
"#,
        )
        .unwrap();

        assert_eq!(
            config.workspaces.len(),
            2,
            "Expected exactly two workspaces"
        );

        let first = &config.workspaces[0];
        assert_eq!(first.dir, "/path/to/directory");
        assert_eq!(first.cmd, CommandType::from("some_command"));
        assert_eq!(
            first.ignore.as_ref().unwrap(),
            &vec!["file1.txt", "file2.txt"]
        );
        assert_eq!(first.bin_path.as_deref(), Some("/usr/local/bin/executable"));
        assert_eq!(first.bin_arg.as_ref().unwrap(), &vec!["--arg1", "--arg2"]);

        let second = &config.workspaces[1];
        assert_eq!(second.dir, "/another/directory");
        assert_eq!(
            second.cmd,
            CommandType::Multiple(vec!["command1".to_string(), "command2".to_string()])
        );
        assert_eq!(second.ignore.as_ref().unwrap(), &vec!["file3.txt"]);
        assert!(second.bin_path.is_none());
        assert!(second.bin_arg.is_none());
    }

    #[test]
    fn test_config_from_file_not_found() {
        let err = Config::from_file("/nonexistent/path/to/config.yaml").unwrap_err();

        match err {
            Error::ConfigRead { path, .. } => assert_eq!(path, "/nonexistent/path/to/config.yaml"),
            other => panic!("expected Error::ConfigRead, got {other:?}"),
        }
    }

    #[test]
    fn test_config_from_file_invalid_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rustywatch.yaml");
        fs::write(
            &path,
            "workspaces:\n  - dir: \".\"\n    cmd: [invalid yaml without closing bracket\n",
        )
        .unwrap();

        assert!(matches!(
            Config::from_file(&path).unwrap_err(),
            Error::ConfigParse { .. }
        ));
    }

    #[test]
    fn test_config_from_file_missing_required_fields() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rustywatch.yaml");
        // `dir` is required.
        fs::write(&path, "workspaces:\n  - cmd: \"some_command\"\n").unwrap();

        assert!(matches!(
            Config::from_file(&path).unwrap_err(),
            Error::ConfigParse { .. }
        ));
    }

    #[test]
    fn test_config_from_file_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rustywatch.yaml");
        fs::write(&path, "").unwrap();

        assert!(matches!(
            Config::from_file(&path).unwrap_err(),
            Error::ConfigParse { .. }
        ));
    }

    #[test]
    fn test_workspace_with_multiple_commands() {
        let workspace = Workspace::new("/path/to/dir").cmds(["npm install", "npm start"]);

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
    fn test_command_type_clone() {
        let cmd = CommandType::Single("test".to_string());
        assert_eq!(cmd.clone(), cmd);
    }
}
