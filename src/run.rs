//! Entry points that wire CLI arguments and config files to the [`Watcher`].
//!
//! These are what `main` calls; library users should build a [`Watcher`]
//! directly instead.

use crate::{
    args::Args,
    config::schema::Workspace,
    error::{Error, Result},
    watcher::Watcher,
};

/// Runs RustyWatch from a configuration file.
///
/// Reads and validates the config at `args.config` (defaults to
/// `rustywatch.yaml`, overridable via `--cfg`), then watches every workspace it
/// declares concurrently.
///
/// # Errors
///
/// Returns the config read/parse/validation failure, or the first workspace
/// failure. Unlike previous versions this never terminates the process — the
/// caller decides what to do with the error.
pub async fn config(args: Args) -> Result<()> {
    Watcher::from_config_file(&args.config)?.run().await
}

/// Runs RustyWatch directly from CLI arguments, without a configuration file.
///
/// The result is returned to the caller (`main`) rather than terminating the
/// process here. This keeps the function testable and avoids killing the test
/// harness when it is exercised from unit tests.
///
/// # Errors
///
/// Returns [`Error::InvalidConfig`] if the arguments do not describe a runnable
/// workspace, plus any error produced while setting up or running the watcher.
pub async fn cli(args: Args) -> Result<()> {
    watcher_from_args(args)?.run().await
}

/// Translates parsed CLI arguments into a validated [`Watcher`].
///
/// # Errors
///
/// Returns [`Error::InvalidConfig`] if no command was supplied, and
/// [`Error::Ignore`] if `--ignore` contains an invalid glob.
pub fn watcher_from_args(args: Args) -> Result<Watcher> {
    let dir = args.dir.unwrap_or_else(|| ".".to_string());

    let commands = args.command.unwrap_or_default();
    if commands.iter().all(|cmd| cmd.trim().is_empty()) {
        return Err(Error::invalid_config(
            "no command to run: pass --cmd, or add a rustywatch.yaml (see `rustywatch init`)",
        ));
    }

    let mut workspace = Workspace::new(dir).cmds(commands);

    if let Some(ignore) = args.ignore {
        workspace = workspace.ignore(ignore);
    }
    if let Some(bin_path) = args.bin_path {
        workspace = workspace.bin_path(bin_path);
    }
    if let Some(bin_arg) = args.bin_arg {
        workspace = workspace.bin_arg(bin_arg);
    }

    Watcher::builder().workspace(workspace).build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    fn args() -> Args {
        Args {
            config: "rustywatch.yaml".to_string(),
            dir: Some(".".to_string()),
            command: Some(vec!["echo 'test'".to_string()]),
            ignore: None,
            bin_path: None,
            bin_arg: None,
            monitor: false,
        }
    }

    // `cli` must return `Ok(())` (rather than terminating the process) when the
    // watcher exits under `cfg!(test)`, otherwise this assertion would be dead
    // code and the test binary would be killed mid-run.
    #[test]
    async fn test_cli_returns_ok() {
        assert!(cli(args()).await.is_ok());
    }

    #[test]
    async fn test_config_reports_missing_file() {
        let args = Args {
            config: "/nonexistent/rustywatch.yaml".to_string(),
            ..args()
        };

        assert!(matches!(
            config(args).await.unwrap_err(),
            Error::ConfigRead { .. }
        ));
    }

    #[test]
    async fn test_cli_requires_a_command() {
        let args = Args {
            command: None,
            ..args()
        };

        let err = cli(args).await.unwrap_err();
        assert!(err.to_string().contains("no command to run"));
    }

    #[test]
    async fn test_watcher_from_args_maps_every_flag() {
        let args = Args {
            ignore: Some(vec!["target/".to_string()]),
            bin_path: Some("target/debug/app".to_string()),
            bin_arg: Some(vec!["--port".to_string(), "8080".to_string()]),
            ..args()
        };

        let watcher = watcher_from_args(args).unwrap();
        let workspace = &watcher.workspaces()[0];

        assert_eq!(workspace.dir, ".");
        assert_eq!(
            workspace.ignore.as_deref(),
            Some(["target/".to_string()].as_slice())
        );
        assert_eq!(workspace.bin_path.as_deref(), Some("target/debug/app"));
        assert_eq!(
            workspace.bin_arg.as_deref(),
            Some(["--port".to_string(), "8080".to_string()].as_slice())
        );
    }

    #[test]
    async fn test_watcher_from_args_rejects_bad_ignore_glob() {
        let args = Args {
            ignore: Some(vec!["src/**/[".to_string()]),
            ..args()
        };

        assert!(matches!(
            watcher_from_args(args).unwrap_err(),
            Error::Ignore { .. }
        ));
    }
}
