use std::{
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
};
use tokio::task;

pub async fn exec(cmd: String) -> Result<Child, Box<dyn std::error::Error>> {
    let output = task::spawn_blocking(move || {
        if cfg!(windows) {
            Command::new("cmd")
                .arg("/C")
                .arg(cmd)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
        }
    })
    .await?;

    Ok(output)
}

pub fn cmd_result(child: Child) {
    macro_rules! process_output {
        ($reader:expr, $print_fn:ident) => {
            for line in $reader.lines() {
                $print_fn!("{}", line.unwrap());
            }
        };
    }

    let stdout = child.stdout.unwrap();
    let stderr = child.stderr.unwrap();
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    process_output!(stdout_reader, println);
    process_output!(stderr_reader, eprintln);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(not(windows))]
    async fn test_exec_unix() {
        let result = exec("echo 'Hello, World!'".to_string()).await;
        assert!(result.is_ok());

        let child = result.unwrap();
        let output = child.wait_with_output().unwrap();

        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "Hello, World!"
        );
    }

    #[tokio::test]
    #[cfg(windows)]
    async fn test_exec_windows() {
        let result = exec("echo Hello, World!".to_string()).await;
        assert!(result.is_ok());

        let child = result.unwrap();
        let output = child.wait_with_output().unwrap();

        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "Hello, World!"
        );
    }
}
