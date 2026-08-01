use std::process::Command;

#[test]
fn test_run_cli_in_thread() {
    let args = vec![
        "-h", // help
        "-V", // version
    ];

    // Run each invocation to completion and assert it exits successfully,
    // reaping the child process instead of leaving it as a zombie.
    for arg in args {
        let output = Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg(arg)
            .output()
            .expect("Failed to start the CLI");

        assert!(
            output.status.success(),
            "`{}` should exit successfully",
            arg
        );
    }
}
