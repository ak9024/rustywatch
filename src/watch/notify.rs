use crate::{
    config::schema::CommandType,
    watch::{filter::is_ignored, reload::reload},
};
use log::{error, info, warn};
use notify::{event::ModifyKind, recommended_watcher, Event, EventKind, RecursiveMode, Watcher};
use std::{
    process::{self, exit, Child},
    result::Result,
    sync::mpsc::channel,
};

pub async fn watcher(
    dir: String,
    cmd: CommandType,
    ignore: Option<Vec<String>>,
    bin_path: Option<String>,
    bin_arg: Option<Vec<String>>,
) -> notify::Result<()> {
    let ignore = ignore.unwrap_or_else(|| {
        warn!("RustyWatch provide options to ignored specific directory or files to be ignored.");
        vec![".git/".to_string()]
    });

    let mut running_binary: Option<Child> = None;

    // @NOTE
    // first time starting reload the binary
    reload(
        &mut running_binary,
        &cmd,
        bin_path.as_ref(),
        bin_arg.as_ref(),
    )
    .await;

    // @NOTE
    // define a channel to store and receive any data process in thread.
    let (tx, rx) = channel();

    let mut watcher = recommended_watcher(move |res: Result<Event, notify::Error>| {
        tx.send(res).unwrap();
    })
    .unwrap();

    // @NOTE
    // listen the directory with recursive mode
    match watcher.watch(dir.as_ref(), RecursiveMode::Recursive) {
        Ok(_) => {
            info!("Waching directory: {:?}", dir);

            // @NOTE
            // in testing env need to skip loop.
            // prevent blocking
            if cfg!(test) {
                warn!("Running in test environment");
                process::exit(0)
            }

            while let Ok(Ok(event)) = rx.recv() {
                // @NOTE
                // if data modified do restarting.
                if let EventKind::Modify(modify_kind) = event.kind {
                    if matches!(modify_kind, ModifyKind::Data(_)) {
                        // @NOTE
                        // filter that paths based on ignore definition
                        let paths = event
                            .paths
                            .iter()
                            .filter(|path| !is_ignored(path, &ignore))
                            .collect::<Vec<_>>();

                        // @NOTE
                        // if paths not empty please reload the binary.
                        if !paths.is_empty() {
                            if let Some(file) = paths.first() {
                                info!("File changed: {:?}", file);
                            }

                            reload(
                                &mut running_binary,
                                &cmd,
                                bin_path.as_ref(),
                                bin_arg.as_ref(),
                            )
                            .await;
                        }
                    }
                }
            }
        }
        Err(e) => {
            error!("Error to watching directory: {:?}", e.paths);
            exit(1)
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::write;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_watch() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().to_str().unwrap().to_string();

        write(temp_dir.path().join("test.txt"), "initial content").unwrap();

        let cmd = CommandType::Single("echo".to_string());
        let ignore = Some(vec![".git".to_string()]);

        let watch_task = tokio::spawn(async move {
            watcher(dir_path, cmd, ignore, None, None).await.unwrap();
        });

        write(temp_dir.path().join("test.txt"), "modified content").unwrap();

        watch_task.abort();
    }

    #[tokio::test]
    async fn test_watch_with_default_ignore() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().to_str().unwrap().to_string();

        write(temp_dir.path().join("test.txt"), "initial content").unwrap();

        let cmd = CommandType::Single("echo".to_string());
        // Pass None to use default ignore patterns
        let ignore: Option<Vec<String>> = None;

        let watch_task = tokio::spawn(async move {
            watcher(dir_path, cmd, ignore, None, None).await.unwrap();
        });

        watch_task.abort();
    }

    #[tokio::test]
    async fn test_watch_with_multiple_commands() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().to_str().unwrap().to_string();

        write(temp_dir.path().join("test.txt"), "initial content").unwrap();

        let cmd = CommandType::Multiple(vec!["echo first".to_string(), "echo second".to_string()]);
        let ignore = Some(vec![".git".to_string()]);

        let watch_task = tokio::spawn(async move {
            watcher(dir_path, cmd, ignore, None, None).await.unwrap();
        });

        watch_task.abort();
    }

    #[tokio::test]
    async fn test_watch_with_bin_path() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().to_str().unwrap().to_string();

        write(temp_dir.path().join("test.txt"), "initial content").unwrap();

        let cmd = CommandType::Single("echo build".to_string());
        let ignore = Some(vec![".git".to_string()]);
        let bin_path = Some("/tmp/test_binary".to_string());

        let watch_task = tokio::spawn(async move {
            watcher(dir_path, cmd, ignore, bin_path, None).await.unwrap();
        });

        watch_task.abort();
    }

    #[tokio::test]
    async fn test_watch_with_bin_args() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().to_str().unwrap().to_string();

        write(temp_dir.path().join("test.txt"), "initial content").unwrap();

        let cmd = CommandType::Single("echo build".to_string());
        let ignore = Some(vec![".git".to_string()]);
        let bin_path = Some("/tmp/test_binary".to_string());
        let bin_arg = Some(vec!["--port".to_string(), "8080".to_string()]);

        let watch_task = tokio::spawn(async move {
            watcher(dir_path, cmd, ignore, bin_path, bin_arg).await.unwrap();
        });

        watch_task.abort();
    }

    #[tokio::test]
    async fn test_watch_with_multiple_ignore_patterns() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().to_str().unwrap().to_string();

        write(temp_dir.path().join("test.txt"), "initial content").unwrap();

        let cmd = CommandType::Single("echo".to_string());
        let ignore = Some(vec![
            ".git".to_string(),
            "node_modules".to_string(),
            "target".to_string(),
            "*.log".to_string(),
        ]);

        let watch_task = tokio::spawn(async move {
            watcher(dir_path, cmd, ignore, None, None).await.unwrap();
        });

        watch_task.abort();
    }
}
