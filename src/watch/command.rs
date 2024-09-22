use std::process::{Child, Stdio};

pub async fn exec(cmd: String) -> Result<Child, Box<dyn std::error::Error>> {
    let output = tokio::task::spawn_blocking(move || {
        if cfg!(windows) {
            std::process::Command::new("cmd")
                .arg("/C")
                .arg(cmd)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap()
        } else {
            std::process::Command::new("sh")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_exec() {
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
}
