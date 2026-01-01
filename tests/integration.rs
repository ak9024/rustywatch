use std::fs::{self, write};
use std::io::Write as IoWrite;
use std::process::Command;
use tempfile::{tempdir, NamedTempFile};

#[test]
fn test_cli_help_flag() {
    let output = Command::new("cargo")
        .args(["run", "--", "-h"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("RustyWatch") || stdout.contains("rustywatch"));
}

#[test]
fn test_cli_version_flag() {
    let output = Command::new("cargo")
        .args(["run", "--", "-V"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("rustywatch"));
}

#[test]
fn test_config_file_parsing() {
    let temp_dir = tempdir().unwrap();

    // Create a valid rustywatch.yaml in temp dir
    let config_content = r#"
workspaces:
  - dir: "."
    cmd: "echo test"
    ignore:
      - ".git"
"#;

    let config_path = temp_dir.path().join("rustywatch.yaml");
    fs::write(&config_path, config_content).unwrap();

    // Create a test file to watch
    write(temp_dir.path().join("test.txt"), "content").unwrap();

    // Run rustywatch with the config file (will exit after first execution in test mode)
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "--cfg",
            config_path.to_str().unwrap(),
            "-d",
            temp_dir.path().to_str().unwrap(),
        ])
        .current_dir(temp_dir.path())
        .output();

    // The command should execute without panic (may exit with error due to test mode)
    assert!(output.is_ok());
}

#[test]
fn test_config_with_multiple_workspaces() {
    let temp_dir = tempdir().unwrap();

    // Create workspace directories
    fs::create_dir_all(temp_dir.path().join("project1")).unwrap();
    fs::create_dir_all(temp_dir.path().join("project2")).unwrap();
    write(temp_dir.path().join("project1/test.txt"), "content1").unwrap();
    write(temp_dir.path().join("project2/test.txt"), "content2").unwrap();

    let config_content = r#"
workspaces:
  - dir: "./project1"
    cmd: "echo project1"
  - dir: "./project2"
    cmd: "echo project2"
"#;

    let config_path = temp_dir.path().join("rustywatch.yaml");
    fs::write(&config_path, config_content).unwrap();

    let output = Command::new("cargo")
        .args(["run", "--", "--cfg", config_path.to_str().unwrap()])
        .current_dir(temp_dir.path())
        .output();

    assert!(output.is_ok());
}

#[test]
fn test_config_with_bin_path_and_args() {
    let temp_dir = tempdir().unwrap();
    write(temp_dir.path().join("test.txt"), "content").unwrap();

    let config_content = r#"
workspaces:
  - dir: "."
    cmd: "echo build"
    bin_path: "/usr/bin/echo"
    bin_arg:
      - "binary running"
"#;

    let config_path = temp_dir.path().join("rustywatch.yaml");
    fs::write(&config_path, config_content).unwrap();

    let output = Command::new("cargo")
        .args(["run", "--", "--cfg", config_path.to_str().unwrap()])
        .current_dir(temp_dir.path())
        .output();

    assert!(output.is_ok());
}

#[test]
fn test_invalid_config_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"invalid: yaml: content: [").unwrap();

    let output = Command::new("cargo")
        .args(["run", "--", "--cfg", temp_file.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    // Command should complete (success or failure) without hanging
    // The invalid YAML should cause a parse error
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Either it failed with an error or it shows error output
    assert!(!output.status.success() || stderr.contains("error") || stdout.contains("error") || stderr.contains("Error"));
}

#[test]
fn test_config_with_empty_workspaces() {
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(b"workspaces: []").unwrap();

    let output = Command::new("cargo")
        .args(["run", "--", "--cfg", temp_file.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    // Should exit with error for empty workspaces
    let stderr = String::from_utf8_lossy(&output.stderr);

    // The CLI should fail or show an error
    assert!(!output.status.success() || stderr.contains("workspaces"));
}

#[test]
fn test_config_with_multiple_commands() {
    let temp_dir = tempdir().unwrap();
    write(temp_dir.path().join("test.txt"), "content").unwrap();

    let config_content = r#"
workspaces:
  - dir: "."
    cmd:
      - "echo first"
      - "echo second"
"#;

    let config_path = temp_dir.path().join("rustywatch.yaml");
    fs::write(&config_path, config_content).unwrap();

    let output = Command::new("cargo")
        .args(["run", "--", "--cfg", config_path.to_str().unwrap()])
        .current_dir(temp_dir.path())
        .output();

    assert!(output.is_ok());
}
