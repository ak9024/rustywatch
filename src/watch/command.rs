use std::collections::HashMap;
use std::io::Error;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

/// Execute a command asynchronously using tokio::process
/// Takes a reference to avoid cloning
/// Accepts optional environment variables to inject
pub async fn exec(cmd: &str, env_vars: &HashMap<String, String>) -> Result<Child, Error> {
    let child = if cfg!(windows) {
        Command::new("cmd")
            .arg("/C")
            .arg(cmd)
            .envs(env_vars)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .envs(env_vars)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    };

    child
}

/// Asynchronously read stdout and stderr concurrently
pub async fn buf_reader_async(mut child: Child) -> std::io::Result<()> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "stdout not captured"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "stderr not captured"))?;

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    // Spawn concurrent tasks for reading stdout and stderr
    let stdout_task = tokio::spawn(async move {
        let mut lines = stdout_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            println!("{}", line);
        }
    });

    let stderr_task = tokio::spawn(async move {
        let mut lines = stderr_reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            eprintln!("{}", line);
        }
    });

    // Wait for both to complete concurrently
    let (stdout_result, stderr_result) = tokio::join!(stdout_task, stderr_task);

    // Handle any join errors
    stdout_result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    stderr_result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Wait for the child process to complete
    child.wait().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(not(windows))]
    async fn test_exec_unix() {
        let env_vars = HashMap::new();
        let result = exec("echo 'Hello, World!'", &env_vars).await;
        assert!(result.is_ok());

        let child = result.unwrap();
        let output = child.wait_with_output().await.unwrap();

        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "Hello, World!"
        );
    }

    #[tokio::test]
    #[cfg(windows)]
    async fn test_exec_windows() {
        let env_vars = HashMap::new();
        let result = exec("echo Hello, World!", &env_vars).await;
        assert!(result.is_ok());

        let child = result.unwrap();
        let output = child.wait_with_output().await.unwrap();

        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "Hello, World!"
        );
    }

    #[tokio::test]
    #[cfg(not(windows))]
    async fn test_buf_reader_async() {
        let env_vars = HashMap::new();
        let child = exec("echo 'test output'", &env_vars).await.unwrap();
        let result = buf_reader_async(child).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[cfg(not(windows))]
    async fn test_exec_with_env_vars() {
        let mut env_vars = HashMap::new();
        env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());

        let result = exec("echo $TEST_VAR", &env_vars).await;
        assert!(result.is_ok());

        let child = result.unwrap();
        let output = child.wait_with_output().await.unwrap();

        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            "test_value"
        );
    }
}
