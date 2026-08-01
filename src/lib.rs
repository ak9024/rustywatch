//! # RustyWatch
//!
//! Live reloading for any programming language, built with Rust.
//!
//! RustyWatch watches one or more workspace directories for file changes and
//! re-runs a build/run command (optionally restarting a compiled binary) each
//! time a relevant file is modified. It can be driven either by CLI arguments
//! or by a `rustywatch.yaml` configuration file, and ships with an optional
//! terminal UI process monitor (`--monitor`).
//!
//! ## Module overview
//!
//! - [`args`] — command-line argument and subcommand definitions (via `clap`).
//! - [`config`] — configuration schema, file loading and `.env` support.
//! - [`init`] — the interactive `init` command that scaffolds a config file.
//! - [`logger`] — logging initialization.
//! - [`monitor`] — the `--monitor` terminal UI dashboard.
//! - [`run`] — entry points that wire arguments/config to the file watcher.
//! - [`watch`] — the file-watching engine: event filtering, debouncing and
//!   non-blocking reloads.

/// Command-line argument parsing and subcommand definitions.
pub mod args;
/// Configuration schema, YAML loading and environment-file support.
pub mod config;
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
