use crate::{
    args::Args,
    config::{env_loader::load_env_file, helper::read, schema::CommandType},
    watch::notify as watch_notify,
};
use futures::future::join_all;
use notify::Error as NotifyError;
use std::{collections::HashMap, error::Error, process};
use watch_notify::watcher;

/// Runs RustyWatch from a configuration file.
///
/// Reads and validates the config at `args.config` (defaults to
/// `rustywatch.yaml`, overridable via `--cfg`), then spawns one asynchronous
/// task per workspace. Any workspace that fails causes the process to exit with
/// a non-zero status.
///
/// # Errors
///
/// Returns an error if the configuration cannot be read or fails validation.
pub async fn config(args: Args) -> Result<(), Box<dyn Error>> {
    match read(args.config) {
        Ok(config) => match config.validate() {
            Ok(_) => {
                let tasks = config.workspaces.into_iter().map(|workspace| {
                    // @NOTE
                    // Load environment variables from env_file if specified
                    // Path resolution:
                    // - Starts with '/': absolute path from root (remove leading /)
                    // - Otherwise: relative to workspace dir
                    let env_vars = match &workspace.env_file {
                        Some(env_file) => {
                            let resolved_path = if env_file.starts_with('/') {
                                env_file.trim_start_matches('/').to_string()
                            } else {
                                format!("{}/{}", workspace.dir, env_file)
                            };
                            load_env_file(&resolved_path)
                        }
                        None => HashMap::new(),
                    };

                    // @NOTE
                    // all workspace run inside thread as a multi thread.
                    // using move to transfer ownership between thread.
                    // then thread running async
                    tokio::spawn(async move {
                        run(
                            workspace.dir,
                            workspace.cmd,
                            workspace.ignore,
                            workspace.bin_path,
                            workspace.bin_arg,
                            env_vars,
                        )
                        .await
                    })
                });

                let results = join_all(tasks).await;

                // @NOTE
                // results can be return after that value have done in thread.
                // if the result is ok, the process still continue.
                // any result must be exit.
                for result in results {
                    match result {
                        Ok(Ok(_)) => continue,
                        _ => {
                            process::exit(1);
                        }
                    }
                }

                Ok(())
            }
            Err(e) => Err(e.into()),
        },
        Err(e) => Err(e),
    }
}

/// Runs RustyWatch directly from CLI arguments, without a configuration file.
///
/// The result is returned to the caller (`main`) rather than terminating the
/// process here. This keeps the function testable and avoids killing the test
/// harness when it is exercised from unit tests.
///
/// # Errors
///
/// Returns any error produced while setting up or running the file watcher.
pub async fn cli(args: Args) -> Result<(), NotifyError> {
    let dir = args.dir.unwrap_or_else(|| ".".to_string());
    let cmd = match args.command {
        Some(command) => CommandType::Multiple(command),
        None => CommandType::Single(String::new()),
    };
    let env_vars = HashMap::new();

    run(dir, cmd, args.ignore, args.bin_path, args.bin_arg, env_vars).await
}

/// Thin wrapper around the file watcher that starts watching `dir` and reloads
/// using the given command, ignore patterns, binary and environment variables.
pub async fn run(
    dir: String,
    cmd: CommandType,
    ignore: Option<Vec<String>>,
    bin_path: Option<String>,
    bin_arg: Option<Vec<String>>,
    env_vars: HashMap<String, String>,
) -> Result<(), NotifyError> {
    match watcher(dir, cmd, ignore, bin_path, bin_arg, env_vars).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    // `cli` must return `Ok(())` (rather than terminating the process) when the
    // watcher exits under `cfg!(test)`, otherwise this assertion would be dead
    // code and the test binary would be killed mid-run.
    #[test]
    async fn test_cli_returns_ok() {
        let args = Args {
            config: "".to_string(),
            dir: Some(".".to_string()),
            command: Some(vec!["echo 'test'".to_string()]),
            ignore: None,
            bin_path: None,
            bin_arg: None,
            monitor: false,
        };

        let result = cli(args).await;
        assert!(result.is_ok());
    }

    #[test]
    async fn test_run() {
        let env_vars = HashMap::new();
        let result = run(
            ".".to_string(),
            CommandType::Single("echo 'test'".to_string()),
            None,
            None,
            None,
            env_vars,
        )
        .await;

        assert!(result.is_ok());
    }
}
