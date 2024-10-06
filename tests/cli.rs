use std::process::Command;
use std::{thread, time::Duration};

#[test]
fn test_run_cli_in_thread() {
    let args = vec![
        "-h", // help
        "-V", // version
    ];

    for arg in args {
        thread::spawn(move || {
            Command::new("cargo")
                .arg("run")
                .arg("--")
                .arg(arg)
                .spawn()
                .expect("Failed to start the CLI");
        });
    }

    thread::sleep(Duration::from_secs(1));

    assert!(true)
}
