//! The primary entry point of the library API: [`Watcher`] and
//! [`WatcherBuilder`].
//!
//! A [`Watcher`] owns one or more [`Workspace`]s and drives the whole watch
//! pipeline for each of them concurrently. Build one in code:
//!
//! ```no_run
//! use rustywatch::{Watcher, Workspace};
//!
//! # async fn demo() -> rustywatch::Result<()> {
//! Watcher::builder()
//!     .workspace(
//!         Workspace::new("./api")
//!             .cmd("cargo build")
//!             .bin_path("target/debug/api"),
//!     )
//!     .workspace(Workspace::new("./web").cmd("npm run dev"))
//!     .build()?
//!     .run()
//!     .await
//! # }
//! ```
//!
//! …or load the same `rustywatch.yaml` the CLI uses:
//!
//! ```no_run
//! # async fn demo() -> rustywatch::Result<()> {
//! rustywatch::Watcher::from_config_file("rustywatch.yaml")?
//!     .run()
//!     .await
//! # }
//! ```

use crate::config::schema::{Config, Workspace};
use crate::error::{Error, Result};
use crate::watch::debouncer::DebouncerConfig;
use crate::watch::notify::watch;
use futures::future::select_all;
use log::error;
use std::path::Path;
use std::time::Duration;
use tokio::task::JoinHandle;

/// A configured set of workspaces, ready to run.
///
/// Construct one with [`Watcher::builder`], [`Watcher::from_config`] or
/// [`Watcher::from_config_file`]. Every constructor validates up front, so
/// [`run`](Self::run) only fails on genuine runtime problems.
#[derive(Debug, Clone)]
pub struct Watcher {
    workspaces: Vec<Workspace>,
    debounce: DebouncerConfig,
}

impl Watcher {
    /// Starts building a watcher workspace by workspace.
    pub fn builder() -> WatcherBuilder {
        WatcherBuilder::new()
    }

    /// Builds a watcher from an already-parsed [`Config`].
    ///
    /// # Errors
    ///
    /// Returns the first failure from [`Config::validate`].
    pub fn from_config(config: Config) -> Result<Self> {
        config.validate()?;

        Ok(Self {
            workspaces: config.workspaces,
            debounce: DebouncerConfig::default(),
        })
    }

    /// Reads, parses and validates a `rustywatch.yaml`-style config file.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ConfigRead`] if the file is missing or unreadable,
    /// [`Error::ConfigParse`] if it is not valid RustyWatch YAML, and any
    /// validation failure from [`Config::validate`].
    pub fn from_config_file(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_config(Config::from_file(path)?)
    }

    /// The workspaces this watcher will run.
    pub fn workspaces(&self) -> &[Workspace] {
        &self.workspaces
    }

    /// The debounce settings applied to every workspace.
    pub fn debounce(&self) -> &DebouncerConfig {
        &self.debounce
    }

    /// Runs every workspace concurrently until one fails.
    ///
    /// Each workspace gets its own task. The first failure aborts the remaining
    /// workspaces and is returned — unlike the CLI, nothing here terminates the
    /// process, so this is safe to call from a larger application.
    ///
    /// Under `cfg(test)` each workspace returns as soon as its directory is
    /// registered, so this resolves to `Ok(())` instead of blocking forever.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Workspace`] wrapping the cause of the first workspace
    /// that failed, or [`Error::Task`] if a workspace task panicked.
    pub async fn run(self) -> Result<()> {
        let debounce = self.debounce;

        let mut handles: Vec<JoinHandle<Result<()>>> = self
            .workspaces
            .into_iter()
            .map(|workspace| {
                let debounce = debounce.clone();
                tokio::spawn(async move {
                    let dir = workspace.dir.clone();
                    watch(workspace, debounce)
                        .await
                        .map_err(|e| Error::workspace(dir, e))
                })
            })
            .collect();

        while !handles.is_empty() {
            let (joined, _index, rest) = select_all(handles).await;

            match joined.map_err(Error::Task).and_then(|result| result) {
                // A workspace finished cleanly; keep the others going.
                Ok(()) => handles = rest,
                Err(e) => {
                    error!("{}", e);
                    for handle in rest {
                        handle.abort();
                    }
                    return Err(e);
                }
            }
        }

        Ok(())
    }
}

/// Incremental builder for [`Watcher`].
///
/// See the [module docs](self) for a full example.
#[derive(Debug, Clone, Default)]
pub struct WatcherBuilder {
    workspaces: Vec<Workspace>,
    debounce: DebouncerConfig,
}

impl WatcherBuilder {
    /// Creates an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds one workspace.
    pub fn workspace(mut self, workspace: Workspace) -> Self {
        self.workspaces.push(workspace);
        self
    }

    /// Adds several workspaces.
    pub fn workspaces<I: IntoIterator<Item = Workspace>>(mut self, workspaces: I) -> Self {
        self.workspaces.extend(workspaces);
        self
    }

    /// Adds every workspace from `config`, keeping any already added.
    pub fn config(self, config: Config) -> Self {
        self.workspaces(config.workspaces)
    }

    /// Quiet period after the last file event before a reload fires.
    ///
    /// Defaults to 300ms.
    pub fn debounce_delay(mut self, delay: Duration) -> Self {
        self.debounce.debounce_delay = delay;
        self
    }

    /// Hard ceiling on how long a reload can be deferred by a continuous stream
    /// of file events.
    ///
    /// Defaults to 2s.
    pub fn max_debounce_delay(mut self, delay: Duration) -> Self {
        self.debounce.max_delay = delay;
        self
    }

    /// Validates the accumulated workspaces and produces a [`Watcher`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidConfig`] if no workspaces were added, or if a
    /// workspace has an empty `dir` or no command, and [`Error::Ignore`] if an
    /// ignore pattern is not a valid glob.
    pub fn build(self) -> Result<Watcher> {
        let config = Config::new(self.workspaces);
        config.validate()?;

        Ok(Watcher {
            workspaces: config.workspaces,
            debounce: self.debounce,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_builder_requires_a_workspace() {
        let err = Watcher::builder().build().unwrap_err();

        assert!(matches!(err, Error::InvalidConfig(_)));
        assert!(err.to_string().contains("`workspaces` must not be empty"));
    }

    #[test]
    fn test_builder_collects_workspaces() {
        let watcher = Watcher::builder()
            .workspace(Workspace::new("./api").cmd("cargo build"))
            .workspaces([Workspace::new("./web").cmd("npm run dev")])
            .build()
            .unwrap();

        let dirs: Vec<_> = watcher
            .workspaces()
            .iter()
            .map(|w| w.dir.as_str())
            .collect();
        assert_eq!(dirs, vec!["./api", "./web"]);
    }

    #[test]
    fn test_builder_overrides_debounce() {
        let watcher = Watcher::builder()
            .workspace(Workspace::new(".").cmd("echo"))
            .debounce_delay(Duration::from_millis(50))
            .max_debounce_delay(Duration::from_millis(500))
            .build()
            .unwrap();

        assert_eq!(watcher.debounce().debounce_delay, Duration::from_millis(50));
        assert_eq!(watcher.debounce().max_delay, Duration::from_millis(500));
    }

    #[test]
    fn test_builder_rejects_invalid_workspace() {
        let err = Watcher::builder()
            .workspace(Workspace::new("./api"))
            .build()
            .unwrap_err();

        assert!(err.to_string().contains("has no command to run"));
    }

    #[test]
    fn test_builder_accepts_config() {
        let config = Config::new([Workspace::new("./api").cmd("cargo build")]);
        let watcher = Watcher::builder().config(config).build().unwrap();

        assert_eq!(watcher.workspaces().len(), 1);
    }

    #[test]
    fn test_from_config_file_reports_missing_file() {
        let err = Watcher::from_config_file("/nonexistent/rustywatch.yaml").unwrap_err();

        assert!(matches!(err, Error::ConfigRead { .. }));
    }

    #[test]
    fn test_from_config_file_reads_yaml() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("rustywatch.yaml");
        fs::write(
            &path,
            "workspaces:\n  - dir: \".\"\n    cmd: \"echo hello\"\n",
        )
        .unwrap();

        let watcher = Watcher::from_config_file(&path).unwrap();
        assert_eq!(watcher.workspaces()[0].dir, ".");
    }

    #[test]
    fn test_from_config_file_reports_invalid_yaml() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("rustywatch.yaml");
        fs::write(&path, "workspaces: [").unwrap();

        let err = Watcher::from_config_file(&path).unwrap_err();
        assert!(matches!(err, Error::ConfigParse { .. }));
    }

    // Under `cfg(test)` each workspace returns as soon as it is registered, so
    // this exercises the join/abort logic without blocking forever.
    #[tokio::test]
    async fn test_run_completes_for_every_workspace() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();

        let watcher = Watcher::builder()
            .workspace(Workspace::new(path).cmd("echo one"))
            .workspace(Workspace::new(path).cmd("echo two"))
            .build()
            .unwrap();

        assert!(watcher.run().await.is_ok());
    }

    #[tokio::test]
    async fn test_run_reports_failing_workspace() {
        let err = Watcher::builder()
            .workspace(Workspace::new("/nonexistent/rustywatch/dir").cmd("echo"))
            .build()
            .unwrap()
            .run()
            .await
            .unwrap_err();

        match err {
            Error::Workspace { dir, source } => {
                assert_eq!(dir, "/nonexistent/rustywatch/dir");
                assert!(matches!(*source, Error::Watch(_)));
            }
            other => panic!("expected Error::Workspace, got {other:?}"),
        }
    }
}
