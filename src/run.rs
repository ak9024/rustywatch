use crate::{
    args::Args,
    config::{helper::read, schema::CommandType},
    watch::notify as watch_notify,
};
use futures::future::join_all;
use notify::Error as NotifyError;
use std::{error::Error, process};
use watch_notify::watcher;

// @NOTE
// config is an option to run rustywatch with that configuration.
// config can be define with custom name with the cli options like this:
// rustywatch --config custom_config.yaml
// but by default the configuration read from `rustywatch.yaml`
pub async fn config(args: Args) -> Result<(), Box<dyn Error>> {
    match read(args.config) {
        Ok(config) => match config.validate() {
            Ok(_) => {
                let tasks = config.workspaces.into_iter().map(|workspace| {
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

// @NOTE
// cli is an options to run rustywatch without configuration.
// cli take arguments from cli options.
pub async fn cli(args: Args) -> Result<(), NotifyError> {
    let dir = args.dir.unwrap_or_else(|| ".".to_string());
    let cmd = match args.command {
        Some(command) => CommandType::Multiple(command),
        None => CommandType::Single(String::new()),
    };

    match run(dir, cmd, args.ignore, args.bin_path, args.bin_arg).await {
        Ok(_) => process::exit(0),
        Err(_) => process::exit(1),
    }
}

// @NOTE
// run as a wrapper for watcher.
pub async fn run(
    dir: String,
    cmd: CommandType,
    ignore: Option<Vec<String>>,
    bin_path: Option<String>,
    bin_arg: Option<Vec<String>>,
) -> Result<(), NotifyError> {
    match watcher(dir, cmd, ignore, bin_path, bin_arg).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn config() {
        let args = Args {
            config: "".to_string(),
            dir: Some(".".to_string()),
            command: Some(vec!["echo 'test'".to_string()]),
            ignore: None,
            bin_path: None,
            bin_arg: None,
        };

        let result = cli(args).await;
        assert!(result.is_ok());
    }

    #[test]
    async fn test_run() {
        let result = run(
            ".".to_string(),
            CommandType::Single("echo 'test'".to_string()),
            None,
            None,
            None,
        )
        .await;

        assert!(result.is_ok());
    }
}
