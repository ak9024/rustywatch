use crate::{
    config::schema::Workspace,
    error::Result,
    watch::{
        debouncer::{Debouncer, DebouncerConfig},
        filter::CompiledFilter,
        reload_controller::ReloadController,
    },
};
use log::{info, warn};
use notify::{
    event::ModifyKind, recommended_watcher, Event, EventKind, RecursiveMode,
    Watcher as NotifyWatcher,
};
use std::{path::PathBuf, sync::mpsc::channel};
use tokio::sync::mpsc as tokio_mpsc;

/// Watches a single workspace until the process is stopped.
///
/// The pipeline is:
///
/// ```text
/// notify::recommended_watcher (std::sync::mpsc callback)
///   -> std::thread bridge: keeps only EventKind::Modify(ModifyKind::Data(_)),
///      drops paths matching the workspace ignore patterns
///   -> Debouncer (quiet period + hard max) - dedupes paths into one batch
///   -> ReloadController::request_reload()
/// ```
///
/// Only *data modification* events trigger reloads — file creation, deletion
/// and rename do not.
///
/// # Errors
///
/// Returns [`Error::Ignore`](crate::Error::Ignore) if an ignore pattern is not
/// a valid glob, and [`Error::Watch`](crate::Error::Watch) if the platform
/// watcher cannot be created or cannot observe `workspace.dir`.
pub async fn watch(workspace: Workspace, debounce: DebouncerConfig) -> Result<()> {
    let filter = CompiledFilter::new(&workspace.ignore_patterns())?;
    let env_vars = workspace.env_vars();
    let dir = workspace.dir.clone();

    // Create reload controller for non-blocking reloads.
    // The workspace `dir` doubles as the working directory for every command.
    let controller = ReloadController::new(workspace, env_vars);

    // Initial reload (blocking for startup)
    controller.initial_reload().await;

    // Create debouncer for batching file changes
    let (debouncer, mut debounce_rx) = Debouncer::new(debounce);

    // Create tokio channel for forwarding events from the watcher thread
    let (event_tx, mut event_rx) = tokio_mpsc::channel::<Vec<PathBuf>>(100);

    // Define a channel to receive file system events (std::sync for notify callback)
    let (tx, rx) = channel();

    let mut watcher =
        recommended_watcher(move |res: std::result::Result<Event, notify::Error>| {
            // The receiver is dropped once this function returns; ignore the error
            // rather than panicking inside the platform watcher callback.
            let _ = tx.send(res);
        })?;

    // Spawn a blocking task to forward events from std::sync channel to tokio channel
    std::thread::spawn(move || {
        while let Ok(Ok(event)) = rx.recv() {
            if let EventKind::Modify(modify_kind) = event.kind {
                if matches!(modify_kind, ModifyKind::Data(_)) {
                    let filtered_paths: Vec<_> = event
                        .paths
                        .into_iter()
                        .filter(|path| !filter.is_ignored(path))
                        .collect();

                    if !filtered_paths.is_empty() {
                        let _ = event_tx.blocking_send(filtered_paths);
                    }
                }
            }
        }
    });

    // Listen to the directory with recursive mode
    watcher.watch(dir.as_ref(), RecursiveMode::Recursive)?;
    info!("Watching directory: {:?}", dir);

    // In testing env, skip the blocking event loop and return cleanly. Using
    // `process::exit` here would terminate the entire test binary mid-run,
    // silently skipping any tests that had not yet completed on other threads.
    if cfg!(test) {
        warn!("Running in test environment");
        return Ok(());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use std::fs::write;
    use tempfile::tempdir;

    fn temp_workspace() -> (tempfile::TempDir, String) {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().to_str().unwrap().to_string();
        write(temp_dir.path().join("test.txt"), "initial content").unwrap();
        (temp_dir, dir_path)
    }

    #[tokio::test]
    async fn test_watch() {
        let (temp_dir, dir_path) = temp_workspace();

        let workspace = Workspace::new(dir_path).cmd("echo").ignore([".git"]);
        let result = watch(workspace, DebouncerConfig::default()).await;

        write(temp_dir.path().join("test.txt"), "modified content").unwrap();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_watch_with_default_ignore() {
        let (_temp_dir, dir_path) = temp_workspace();

        let workspace = Workspace::new(dir_path).cmd("echo");
        assert!(watch(workspace, DebouncerConfig::default()).await.is_ok());
    }

    #[tokio::test]
    async fn test_watch_with_multiple_commands() {
        let (_temp_dir, dir_path) = temp_workspace();

        let workspace = Workspace::new(dir_path).cmds(["echo first", "echo second"]);
        assert!(watch(workspace, DebouncerConfig::default()).await.is_ok());
    }

    #[tokio::test]
    async fn test_watch_with_bin_path_and_args() {
        let (_temp_dir, dir_path) = temp_workspace();

        let workspace = Workspace::new(dir_path)
            .cmd("echo build")
            .bin_path("/tmp/test_binary")
            .bin_arg(["--port", "8080"]);

        assert!(watch(workspace, DebouncerConfig::default()).await.is_ok());
    }

    #[tokio::test]
    async fn test_watch_with_multiple_ignore_patterns() {
        let (_temp_dir, dir_path) = temp_workspace();

        let workspace = Workspace::new(dir_path).cmd("echo").ignore([
            ".git",
            "node_modules",
            "target",
            "*.log",
        ]);

        assert!(watch(workspace, DebouncerConfig::default()).await.is_ok());
    }

    #[tokio::test]
    async fn test_watch_rejects_invalid_ignore_pattern() {
        let (_temp_dir, dir_path) = temp_workspace();

        let workspace = Workspace::new(dir_path).cmd("echo").ignore(["src/**/["]);
        let err = watch(workspace, DebouncerConfig::default())
            .await
            .unwrap_err();

        assert!(matches!(err, Error::Ignore { .. }));
    }

    #[tokio::test]
    async fn test_watch_reports_missing_directory() {
        let workspace = Workspace::new("/nonexistent/rustywatch/dir").cmd("echo");
        let err = watch(workspace, DebouncerConfig::default())
            .await
            .unwrap_err();

        assert!(matches!(err, Error::Watch(_)));
    }
}
