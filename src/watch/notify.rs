use crate::{
    config::schema::CommandType,
    watch::{
        debouncer::{Debouncer, DebouncerConfig},
        filter::is_ignored,
        ignore_defaults::merge_with_defaults,
        reload_controller::ReloadController,
    },
};
use log::{error, info, warn};
use notify::{event::ModifyKind, recommended_watcher, Event, EventKind, RecursiveMode, Watcher};
use std::{
    collections::HashMap,
    process::{self, exit},
    result::Result,
    sync::mpsc::channel,
};
use tokio::sync::mpsc as tokio_mpsc;

pub async fn watcher(
    dir: String,
    cmd: CommandType,
    ignore: Option<Vec<String>>,
    bin_path: Option<String>,
    bin_arg: Option<Vec<String>>,
    env_vars: HashMap<String, String>,
) -> notify::Result<()> {
    let ignore = merge_with_defaults(ignore);

    // Create reload controller for non-blocking reloads
    // Pass dir so commands execute in the workspace directory
    let controller = ReloadController::new(cmd, bin_path, bin_arg, env_vars, dir.clone());

    // Initial reload (blocking for startup)
    controller.initial_reload().await;

    // Create debouncer for batching file changes
    let debouncer_config = DebouncerConfig::default();
    let (debouncer, mut debounce_rx) = Debouncer::new(debouncer_config);

    // Create tokio channel for forwarding events from the watcher thread
    let (event_tx, mut event_rx) = tokio_mpsc::channel::<Vec<std::path::PathBuf>>(100);

    // Define a channel to receive file system events (std::sync for notify callback)
    let (tx, rx) = channel();

    let mut watcher = recommended_watcher(move |res: Result<Event, notify::Error>| {
        tx.send(res).unwrap();
    })
    .unwrap();

    // Spawn a blocking task to forward events from std::sync channel to tokio channel
    let ignore_clone = ignore.clone();
    std::thread::spawn(move || {
        while let Ok(Ok(event)) = rx.recv() {
            if let EventKind::Modify(modify_kind) = event.kind {
                if matches!(modify_kind, ModifyKind::Data(_)) {
                    let filtered_paths: Vec<_> = event
                        .paths
                        .into_iter()
                        .filter(|path| !is_ignored(path, &ignore_clone))
                        .collect();

                    if !filtered_paths.is_empty() {
                        let _ = event_tx.blocking_send(filtered_paths);
                    }
                }
            }
        }
    });

    // Listen to the directory with recursive mode
    match watcher.watch(dir.as_ref(), RecursiveMode::Recursive) {
        Ok(_) => {
            info!("Watching directory: {:?}", dir);

            // In testing env, skip the loop to prevent blocking
            if cfg!(test) {
                warn!("Running in test environment");
                process::exit(0)
            }

            // Main event loop using tokio::select! for concurrent handling
            loop {
                tokio::select! {
                    // Handle debounced file change batches
                    Some(batch) = debounce_rx.recv() => {
                        if !batch.is_empty() {
                            if let Some(file) = batch.first() {
                                info!("Files changed ({}): {:?}", batch.len(), file);
                            }
                            // Non-blocking reload request
                            controller.request_reload().await;
                        }
                    }
                    // Handle forwarded file system events
                    Some(paths) = event_rx.recv() => {
                        for path in paths {
                            debouncer.add_path(path).await;
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

    #[allow(unreachable_code)]
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
        let env_vars = HashMap::new();

        let watch_task = tokio::spawn(async move {
            watcher(dir_path, cmd, ignore, None, None, env_vars)
                .await
                .unwrap();
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
        let env_vars = HashMap::new();

        let watch_task = tokio::spawn(async move {
            watcher(dir_path, cmd, ignore, None, None, env_vars)
                .await
                .unwrap();
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
        let env_vars = HashMap::new();

        let watch_task = tokio::spawn(async move {
            watcher(dir_path, cmd, ignore, None, None, env_vars)
                .await
                .unwrap();
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
        let env_vars = HashMap::new();

        let watch_task = tokio::spawn(async move {
            watcher(dir_path, cmd, ignore, bin_path, None, env_vars)
                .await
                .unwrap();
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
        let env_vars = HashMap::new();

        let watch_task = tokio::spawn(async move {
            watcher(dir_path, cmd, ignore, bin_path, bin_arg, env_vars)
                .await
                .unwrap();
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
        let env_vars = HashMap::new();

        let watch_task = tokio::spawn(async move {
            watcher(dir_path, cmd, ignore, None, None, env_vars)
                .await
                .unwrap();
        });

        watch_task.abort();
    }
}
