//! End-to-end tests for the `rustywatch init` subcommand.
//!
//! These drive the compiled binary (via `CARGO_BIN_EXE_rustywatch`) rather than
//! spawning a nested `cargo run`, so they stay fast and do not contend on the
//! build lock. Every invocation passes `--yes`: `init` without it requires a
//! terminal, which a test harness never provides.

use rustywatch::Config;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::{tempdir, TempDir};

const BIN: &str = env!("CARGO_BIN_EXE_rustywatch");

fn run_init(dir: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .arg("init")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to run rustywatch init")
}

fn stdout_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn combined(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// A monorepo with a Rust root, a Go service, a Node app and a decoy project
/// buried inside `node_modules`.
fn monorepo() -> TempDir {
    let root = tempdir().unwrap();
    let path = root.path();

    fs::create_dir_all(path.join("services/api")).unwrap();
    fs::create_dir_all(path.join("web")).unwrap();
    fs::create_dir_all(path.join("node_modules/decoy")).unwrap();

    fs::write(path.join("Cargo.toml"), "[package]\nname = \"shop\"\n").unwrap();
    fs::write(path.join("services/api/go.mod"), "module example.com/api").unwrap();
    fs::write(
        path.join("web/package.json"),
        r#"{"name":"storefront","scripts":{"build":"tsc","serve":"vite preview"}}"#,
    )
    .unwrap();
    fs::write(path.join("web/.env"), "PORT=8080\n").unwrap();
    fs::write(
        path.join("node_modules/decoy/package.json"),
        r#"{"name":"decoy"}"#,
    )
    .unwrap();

    root
}

#[test]
fn test_init_yes_writes_a_loadable_config() {
    let root = tempdir().unwrap();
    fs::create_dir_all(root.path().join("src")).unwrap();
    fs::write(
        root.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\n",
    )
    .unwrap();

    let output = run_init(root.path(), &["--yes"]);
    assert!(output.status.success(), "{}", combined(&output));

    let config_path = root.path().join("rustywatch.yaml");
    let config = Config::from_file(&config_path).expect("generated config must parse");
    config.validate().expect("generated config must validate");

    assert_eq!(config.workspaces.len(), 1);
    let workspace = &config.workspaces[0];

    // The crate root, not `src/` — a relative `bin_path` resolves against the
    // watch directory, so `dir: src` would look for `src/target/debug/demo`.
    assert_eq!(workspace.dir, ".");
    assert_eq!(workspace.bin_path.as_deref(), Some("./target/debug/demo"));
}

#[test]
fn test_init_writes_no_null_fields() {
    let root = tempdir().unwrap();
    fs::write(root.path().join("go.mod"), "module example.com/svc").unwrap();

    let output = run_init(root.path(), &["--yes"]);
    assert!(output.status.success(), "{}", combined(&output));

    let written = fs::read_to_string(root.path().join("rustywatch.yaml")).unwrap();
    assert!(
        !written.contains("null"),
        "unset optional fields must be skipped, got:\n{written}"
    );
}

#[test]
fn test_init_scans_nested_projects() {
    let root = monorepo();

    let output = run_init(root.path(), &["--yes"]);
    assert!(output.status.success(), "{}", combined(&output));

    let config = Config::from_file(root.path().join("rustywatch.yaml")).unwrap();
    let dirs: Vec<_> = config.workspaces.iter().map(|w| w.dir.as_str()).collect();

    assert_eq!(dirs, vec![".", "services/api", "web"]);
    assert!(
        !dirs.iter().any(|d| d.contains("node_modules")),
        "vendored projects must not be picked up"
    );
}

#[test]
fn test_init_reads_package_json_scripts_and_env_file() {
    let root = monorepo();

    let output = run_init(root.path(), &["--yes"]);
    assert!(output.status.success(), "{}", combined(&output));

    let config = Config::from_file(root.path().join("rustywatch.yaml")).unwrap();
    let web = config
        .workspaces
        .iter()
        .find(|w| w.dir == "web")
        .expect("web workspace");

    // No `dev` script exists, so `serve` is the next preference.
    assert_eq!(web.cmd.iter().collect::<Vec<_>>(), vec!["npm run serve"]);
    assert_eq!(web.env_file.as_deref(), Some(".env"));
}

#[test]
fn test_init_depth_zero_scans_only_the_root() {
    let root = monorepo();

    let output = run_init(root.path(), &["--yes", "--depth", "0"]);
    assert!(output.status.success(), "{}", combined(&output));

    let config = Config::from_file(root.path().join("rustywatch.yaml")).unwrap();
    assert_eq!(config.workspaces.len(), 1);
    assert_eq!(config.workspaces[0].dir, ".");
}

#[test]
fn test_init_dir_flag_scopes_the_scan() {
    let root = monorepo();

    let output = run_init(root.path(), &["--yes", "--dir", "services"]);
    assert!(output.status.success(), "{}", combined(&output));

    let config = Config::from_file(root.path().join("rustywatch.yaml")).unwrap();
    assert_eq!(config.workspaces.len(), 1);
    assert_eq!(config.workspaces[0].dir, "api");
}

#[test]
fn test_init_dry_run_prints_without_writing() {
    let root = monorepo();

    let output = run_init(root.path(), &["--yes", "--dry-run"]);
    assert!(output.status.success(), "{}", combined(&output));

    let printed = stdout_of(&output);
    assert!(printed.contains("workspaces:"));
    assert!(printed.contains("services/api"));
    assert!(
        !root.path().join("rustywatch.yaml").exists(),
        "--dry-run must not touch disk"
    );
}

#[test]
fn test_init_yes_refuses_to_overwrite_without_force() {
    let root = tempdir().unwrap();
    fs::write(root.path().join("go.mod"), "module example.com/svc").unwrap();
    fs::write(root.path().join("rustywatch.yaml"), "# hand written\n").unwrap();

    let output = run_init(root.path(), &["--yes"]);

    assert!(!output.status.success());
    assert!(combined(&output).contains("pass --force to overwrite"));
    assert_eq!(
        fs::read_to_string(root.path().join("rustywatch.yaml")).unwrap(),
        "# hand written\n",
        "the existing config must be left alone"
    );
}

#[test]
fn test_init_force_overwrites() {
    let root = tempdir().unwrap();
    fs::write(root.path().join("go.mod"), "module example.com/svc").unwrap();
    fs::write(root.path().join("rustywatch.yaml"), "# hand written\n").unwrap();

    let output = run_init(root.path(), &["--yes", "--force"]);
    assert!(output.status.success(), "{}", combined(&output));

    let written = fs::read_to_string(root.path().join("rustywatch.yaml")).unwrap();
    assert!(written.contains("go build"));
}

#[test]
fn test_init_custom_output_path() {
    let root = tempdir().unwrap();
    fs::write(root.path().join("go.mod"), "module example.com/svc").unwrap();

    let output = run_init(root.path(), &["--yes", "-o", ".rustywatch.yaml"]);
    assert!(output.status.success(), "{}", combined(&output));

    assert!(root.path().join(".rustywatch.yaml").exists());
    assert!(!root.path().join("rustywatch.yaml").exists());
}

#[test]
fn test_init_rejects_missing_scan_directory() {
    let root = tempdir().unwrap();

    let output = run_init(root.path(), &["--yes", "--dir", "does-not-exist"]);

    assert!(!output.status.success());
    assert!(combined(&output).contains("is not a directory"));
}

// Without `--yes`, `init` is interactive. `Command::output` gives the child a
// null stdin, so it must bail out with an explanation instead of hanging.
#[test]
fn test_init_without_yes_requires_a_terminal() {
    let root = tempdir().unwrap();
    fs::write(root.path().join("go.mod"), "module example.com/svc").unwrap();

    let output = run_init(root.path(), &[]);

    assert!(!output.status.success());
    assert!(combined(&output).contains("needs a terminal"));
    assert!(!root.path().join("rustywatch.yaml").exists());
}

#[test]
fn test_init_without_markers_writes_a_placeholder() {
    let root = tempdir().unwrap();

    let output = run_init(root.path(), &["--yes"]);
    assert!(output.status.success(), "{}", combined(&output));

    let config = Config::from_file(root.path().join("rustywatch.yaml")).unwrap();
    assert_eq!(config.workspaces.len(), 1);
    assert_eq!(config.workspaces[0].dir, ".");
    // Still a valid config, just one the user has to edit.
    config.validate().unwrap();
}

#[test]
fn test_init_help_lists_every_flag() {
    let root = tempdir().unwrap();
    let output = run_init(root.path(), &["--help"]);

    assert!(output.status.success());
    let help = stdout_of(&output);

    for flag in [
        "--output",
        "--dir",
        "--depth",
        "--yes",
        "--force",
        "--dry-run",
    ] {
        assert!(help.contains(flag), "`{flag}` missing from init --help");
    }
}
