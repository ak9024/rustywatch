//! Tests the public library surface the way a downstream crate sees it.
//!
//! Only the root re-exports are used here — if something needs a module path,
//! it is not part of the supported API.
//!
//! Nothing in this file calls [`Watcher::run`]: the `cfg!(test)` early return
//! that keeps unit tests from blocking applies to the library's *own* test
//! build, not to an integration test linking the normal build. Calling `run`
//! here would watch forever.

use rustywatch::{
    default_ignore_patterns, CommandType, Config, DebouncerConfig, Error, Watcher, Workspace,
    DEFAULT_IGNORE_PATTERNS,
};
use std::error::Error as _;
use std::fs;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn builder_accepts_a_full_workspace() {
    let watcher = Watcher::builder()
        .workspace(
            Workspace::new("./api")
                .cmd("cargo build")
                .bin_path("target/debug/api")
                .bin_arg(["--port", "8080"])
                .ignore(["target/"])
                .env_file(".env"),
        )
        .workspace(Workspace::new("./web").cmds(["npm install", "npm run dev"]))
        .build()
        .expect("valid workspaces");

    assert_eq!(watcher.workspaces().len(), 2);

    let api = &watcher.workspaces()[0];
    assert_eq!(api.dir, "./api");
    assert_eq!(api.cmd, CommandType::from("cargo build"));
    assert_eq!(api.bin_path.as_deref(), Some("target/debug/api"));
    assert_eq!(api.env_file.as_deref(), Some(".env"));

    let web = &watcher.workspaces()[1];
    assert_eq!(web.cmd.len(), 2);
    assert_eq!(
        web.cmd.iter().collect::<Vec<_>>(),
        vec!["npm install", "npm run dev"]
    );
}

#[test]
fn builder_defaults_debounce_and_allows_overrides() {
    let default = DebouncerConfig::default();
    assert_eq!(default.debounce_delay, Duration::from_millis(300));
    assert_eq!(default.max_delay, Duration::from_secs(2));

    let watcher = Watcher::builder()
        .workspace(Workspace::new(".").cmd("cargo test"))
        .debounce_delay(Duration::from_millis(25))
        .max_debounce_delay(Duration::from_millis(250))
        .build()
        .unwrap();

    assert_eq!(watcher.debounce().debounce_delay, Duration::from_millis(25));
    assert_eq!(watcher.debounce().max_delay, Duration::from_millis(250));
}

#[test]
fn builder_rejects_an_empty_workspace_list() {
    let err = Watcher::builder().build().unwrap_err();

    assert!(matches!(err, Error::InvalidConfig(_)));
    assert!(err.to_string().contains("`workspaces` must not be empty"));
    assert!(err.source().is_none());
}

#[test]
fn builder_rejects_a_workspace_without_a_command() {
    let err = Watcher::builder()
        .workspace(Workspace::new("./api"))
        .build()
        .unwrap_err();

    assert!(err.to_string().contains("has no command to run"));
}

#[test]
fn builder_rejects_a_blank_directory() {
    let err = Watcher::builder()
        .workspace(Workspace::new("   ").cmd("echo hi"))
        .build()
        .unwrap_err();

    assert!(err.to_string().contains("`dir` must not be empty"));
}

// An invalid glob must be reported when the watcher is built, not silently
// swallowed once it is already running.
#[test]
fn builder_rejects_an_invalid_ignore_glob() {
    let err = Watcher::builder()
        .workspace(Workspace::new(".").cmd("echo hi").ignore(["src/**/["]))
        .build()
        .unwrap_err();

    match &err {
        Error::Ignore { pattern, .. } => assert_eq!(pattern.as_deref(), Some("src/**/[")),
        other => panic!("expected Error::Ignore, got {other:?}"),
    }
    assert!(err.source().is_some());
}

#[test]
fn ignore_replaces_defaults_and_extend_ignore_keeps_them() {
    let replaced = Workspace::new(".").cmd("echo").ignore([".git"]);
    assert_eq!(replaced.ignore_patterns(), vec![".git".to_string()]);

    let extended = Workspace::new(".").cmd("echo").extend_ignore(["*.snap"]);
    let patterns = extended.ignore_patterns();
    assert!(patterns.contains(&"target/".to_string()));
    assert!(patterns.contains(&"*.snap".to_string()));

    // Omitting `ignore` entirely also yields the defaults.
    let bare = Workspace::new(".").cmd("echo");
    assert_eq!(bare.ignore_patterns(), default_ignore_patterns());
    assert_eq!(
        default_ignore_patterns().len(),
        DEFAULT_IGNORE_PATTERNS.len()
    );
}

#[test]
fn env_file_resolves_relative_to_the_workspace_dir() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join(".env"), "GREETING=hello\nPORT=\"8080\"\n").unwrap();

    let workspace = Workspace::new(dir.path().to_str().unwrap())
        .cmd("echo")
        .env_file(".env");

    let env = workspace.env_vars();
    assert_eq!(env.get("GREETING"), Some(&"hello".to_string()));
    assert_eq!(env.get("PORT"), Some(&"8080".to_string()));
}

#[test]
fn a_missing_env_file_yields_no_variables() {
    let workspace = Workspace::new(".").cmd("echo").env_file("nope.env");
    assert!(workspace.env_vars().is_empty());
}

#[test]
fn config_round_trips_through_yaml() {
    let config = Config::new([
        Workspace::new("./api").cmd("cargo build").bin_path("./api"),
        Workspace::new("./web").cmds(["npm install", "npm start"]),
    ]);

    let yaml = config.to_yaml().unwrap();
    assert!(!yaml.contains("null"));
    assert_eq!(Config::from_yaml(&yaml).unwrap(), config);
}

#[test]
fn watcher_loads_a_config_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("rustywatch.yaml");
    fs::write(
        &path,
        "workspaces:\n  - dir: \".\"\n    cmd: \"echo hello\"\n    ignore:\n      - \"*.log\"\n",
    )
    .unwrap();

    let watcher = Watcher::from_config_file(&path).unwrap();

    assert_eq!(watcher.workspaces().len(), 1);
    assert_eq!(watcher.workspaces()[0].cmd, CommandType::from("echo hello"));
}

#[test]
fn watcher_reports_a_missing_config_file() {
    let err = Watcher::from_config_file("/nonexistent/rustywatch.yaml").unwrap_err();

    match &err {
        Error::ConfigRead { path, .. } => assert_eq!(path, "/nonexistent/rustywatch.yaml"),
        other => panic!("expected Error::ConfigRead, got {other:?}"),
    }
    // The path and the underlying cause both make it into the message.
    assert!(err.to_string().contains("/nonexistent/rustywatch.yaml"));
    assert!(err.source().is_some());
}

#[test]
fn watcher_reports_unparsable_yaml() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("rustywatch.yaml");
    fs::write(&path, "workspaces: [").unwrap();

    assert!(matches!(
        Watcher::from_config_file(&path).unwrap_err(),
        Error::ConfigParse { .. }
    ));
}

#[test]
fn watcher_validates_a_config_file_it_loads() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("rustywatch.yaml");
    fs::write(&path, "workspaces: []\n").unwrap();

    assert!(matches!(
        Watcher::from_config_file(&path).unwrap_err(),
        Error::InvalidConfig(_)
    ));
}

// `?` into a boxed trait object still works, so callers are not forced to adopt
// `rustywatch::Error` throughout their own code.
#[test]
fn error_converts_into_a_boxed_std_error() {
    fn load() -> Result<Watcher, Box<dyn std::error::Error>> {
        Ok(Watcher::from_config_file("/nonexistent/rustywatch.yaml")?)
    }

    assert!(load().is_err());
}

#[test]
fn command_type_helpers() {
    let single = CommandType::from("cargo build".to_string());
    assert_eq!(single.len(), 1);
    assert!(!single.is_empty());

    let multiple = CommandType::from(vec!["a".to_string(), "b".to_string()]);
    assert_eq!(multiple.len(), 2);
    assert_eq!(multiple.iter().collect::<Vec<_>>(), vec!["a", "b"]);

    assert!(CommandType::from("   ").is_empty());
}
