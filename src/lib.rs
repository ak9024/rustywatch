pub mod args;
pub mod config;
pub mod logger;
pub mod watch;

use config::CommandType;
use std::error::Error;
use watch::notify::watcher;

pub async fn run(
    dir: String,
    cmd: CommandType,
    ignore: Option<Vec<String>>,
    bin_path: Option<String>,
    bin_arg: Option<Vec<String>>,
) -> Result<(), impl Error + Send + Sync> {
    match watcher(dir, cmd, ignore, bin_path, bin_arg).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use crate::config::CommandType;

    #[tokio::test]
    async fn test_run() {
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().to_str().unwrap().to_string();

        let cmd = CommandType::Single("echo 'File changed'".to_string());
        let ignore = Some(vec!["*.tmp".to_string()]);
        let bin_path = Some("/usr/bin/echo".to_string());
        let bin_arg = Some(vec!["Hello".to_string()]);

        let result = super::run(dir_path, cmd, ignore, bin_path, bin_arg).await;

        assert!(result.is_ok());

        temp_dir.close().unwrap();
    }
}
