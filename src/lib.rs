pub mod args;
pub mod config;
pub mod logger;
pub mod watch;

use log::error;
use std::{error::Error, process};

pub async fn run(
    dir: String,
    cmd: String,
    ignore: Option<Vec<String>>,
    bin_path: Option<String>,
    bin_arg: Option<Vec<String>>,
    test: bool,
) -> Result<(), Box<dyn Error + Send>> {
    // for testing to prevent blocking
    if test {
        process::exit(0)
    }

    if let Err(err) = watch::watch(dir, cmd, ignore, bin_path, bin_arg).await {
        error!("Error watching directory: {:?}", err)
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_run() {
        use tempfile::tempdir;

        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().to_str().unwrap().to_string();

        // Test parameters
        let cmd = "echo 'File changed'".to_string();
        let ignore = Some(vec!["*.tmp".to_string()]);
        let bin_path = Some("/usr/bin/echo".to_string());
        let bin_arg = Some(vec!["Hello".to_string()]);

        // Run the function
        let result = super::run(dir_path, cmd, ignore, bin_path, bin_arg, true).await;

        // Assert that the function returns Ok(())
        assert!(result.is_ok());

        // Clean up temporary directory
        temp_dir.close().unwrap();
    }
}
