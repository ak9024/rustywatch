//! # RustyWatch
//!
//! Live reloading for any programming language, built with Rust.
//!
//! RustyWatch watches one or more workspace directories for file changes and
//! re-runs a build/run command (optionally restarting a compiled binary) each
//! time a relevant file is modified. It ships as a CLI, and the same engine is
//! usable as a library.
//!
//! ## Quick start
//!
//! Describe the workspaces in code and run them:
//!
//! ```no_run
//! use rustywatch::{Watcher, Workspace};
//!
//! #[tokio::main]
//! async fn main() -> rustywatch::Result<()> {
//!     Watcher::builder()
//!         .workspace(
//!             Workspace::new("./api")
//!                 .cmd("cargo build")
//!                 .bin_path("target/debug/api")
//!                 .bin_arg(["--port", "8080"])
//!                 .env_file(".env"),
//!         )
//!         .workspace(Workspace::new("./web").cmd("npm run dev"))
//!         .build()?
//!         .run()
//!         .await
//! }
//! ```
//!
//! Or reuse an existing `rustywatch.yaml`:
//!
//! ```no_run
//! # async fn demo() -> rustywatch::Result<()> {
//! rustywatch::Watcher::from_config_file("rustywatch.yaml")?
//!     .run()
//!     .await
//! # }
//! ```
//!
//! [`Watcher::run`] resolves only when a workspace fails: it aborts the
//! remaining workspaces and returns the cause as an [`Error`]. Nothing in this
//! crate calls `process::exit`, so it is safe to embed in a larger application.
//!
//! ## Tuning
//!
//! File events are debounced before a reload fires — 300ms of quiet, capped at
//! 2s under a continuous stream of writes. Override both on the builder:
//!
//! ```no_run
//! # use std::time::Duration;
//! # use rustywatch::{Watcher, Workspace};
//! # fn demo() -> rustywatch::Result<()> {
//! let watcher = Watcher::builder()
//!     .workspace(Workspace::new(".").cmd("cargo test"))
//!     .debounce_delay(Duration::from_millis(50))
//!     .max_debounce_delay(Duration::from_millis(500))
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Things that surprise people
//!
//! - Only *data modification* events trigger reloads — creating, deleting or
//!   renaming a file does not.
//! - [`Workspace::cmds`] runs its commands **in parallel**, not in sequence.
//!   Chain with `&&` inside one command when order matters.
//! - [`Workspace::ignore`] **replaces** the default ignore patterns; use
//!   [`Workspace::extend_ignore`] to keep `target/`, `node_modules/` and the
//!   rest.
//! - Commands run with the workspace `dir` as their working directory, so they
//!   must not be prefixed with `cd`. Relative `bin_path`s resolve against that
//!   same `dir`.
//! - [`Workspace::env_file`] treats a leading `/` as "relative to the project
//!   root", not as a filesystem-absolute path.
//!
//! ## Module overview
//!
//! The types re-exported here are the supported surface; the modules below are
//! public for the CLI's benefit and carry weaker stability guarantees.
//!
//! - [`args`] — command-line argument and subcommand definitions (via `clap`).
//! - [`config`] — configuration schema and `.env` support.
//! - [`error`] — the crate's [`Error`] type and [`Result`] alias.
//! - [`init`] — the interactive `init` command that scaffolds a config file.
//! - [`logger`] — logging initialization.
//! - [`monitor`] — the `--monitor` terminal UI dashboard.
//! - [`run`] — entry points that wire CLI arguments/config to the watcher.
//! - [`watch`] — the file-watching engine: event filtering, debouncing and
//!   non-blocking reloads.
//! - [`watcher`] — [`Watcher`] and [`WatcherBuilder`], the library entry point.

/// Command-line argument parsing and subcommand definitions.
pub mod args;
/// Configuration schema, YAML loading and environment-file support.
pub mod config;
/// Error and result types returned by every fallible entry point.
pub mod error;
/// Interactive `init` command for scaffolding a configuration file.
pub mod init;
/// Logging setup.
pub mod logger;
/// Terminal UI process monitor (`--monitor`).
pub mod monitor;
/// Entry points connecting CLI arguments / config to the watcher.
pub mod run;
/// File-watching engine: filtering, debouncing and reload orchestration.
pub mod watch;
/// The library entry point: [`Watcher`] and [`WatcherBuilder`].
pub mod watcher;

pub use crate::config::schema::{CommandType, Config, Workspace};
pub use crate::error::{Error, Result};
pub use crate::watch::debouncer::DebouncerConfig;
pub use crate::watch::ignore_defaults::{default_ignore_patterns, DEFAULT_IGNORE_PATTERNS};
pub use crate::watcher::{Watcher, WatcherBuilder};
