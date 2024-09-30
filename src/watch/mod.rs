pub mod binary;
pub mod command;
pub mod filter;
pub mod notify;
pub mod reload;

#[cfg(test)]
mod tests {
    use crate::{
        config::CommandType,
        watch::{filter::is_ignored, notify::watch, reload::reload},
    };
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_is_ignored() {
        let ignore_list = vec![".git".to_string(), "target".to_string()];
        assert!(is_ignored(&PathBuf::from(".git"), &ignore_list));
        assert!(is_ignored(&PathBuf::from("target"), &ignore_list));
        assert!(!is_ignored(&PathBuf::from("src"), &ignore_list));
    }

    #[tokio::test]
    async fn test_reload_without_bin_path() {
        let mut running_binary = None;
        let cmd = CommandType::Single("echo 'Hello, World!'".to_string());
        reload(&mut running_binary, &cmd, None, None).await;
        assert!(running_binary.is_none());
    }

    #[tokio::test]
    async fn test_watch_timeout() {
        let dir = tempdir().unwrap();
        let cmd = CommandType::Single("echo 'Test'".to_string());

        let handle = tokio::spawn(async move {
            watch(
                dir.path().to_str().unwrap().to_string(),
                cmd,
                None,
                None,
                None,
            )
            .await
        });
        handle.abort();

        let result = handle.await;
        assert!(result.is_err());
    }
}
