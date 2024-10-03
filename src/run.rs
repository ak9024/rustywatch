use crate::{
    args::Args,
    config::{self, CommandType},
    watch::notify::watcher,
};
use futures::future::join_all;
use log::error;
use std::process;
use validator::Validate;

pub async fn config(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    match config::read(args.config) {
        Ok(config) => match config.validate() {
            Ok(_) => {
                let tasks = config.workspaces.into_iter().map(|workspace| {
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

                for result in results {
                    match result {
                        Ok(Ok(_)) => continue,
                        Ok(Err(e)) => {
                            error!("Task failed: {}", e);
                            process::exit(1);
                        }
                        Err(e) => {
                            error!("Task panicked: {}", e);
                            process::exit(1);
                        }
                    }
                }

                Ok(())
            }
            Err(e) => {
                error!("Config validation failed: {}", e);
                Err(e.into())
            }
        },

        Err(e) => {
            error!("Missing field workspaces at your config: {}", e);
            Err(e)
        }
    }
}

pub async fn cli(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let dir = args.dir.unwrap_or_else(|| ".".to_string());
    let cmd = CommandType::Single(args.command.unwrap_or_default().to_string());

    match run(dir, cmd, args.ignore, args.bin_path, args.bin_arg).await {
        Ok(_) => process::exit(0),
        Err(e) => {
            error!("Error to execute the program: {}", e);
            process::exit(1)
        }
    }
}

pub async fn run(
    dir: String,
    cmd: CommandType,
    ignore: Option<Vec<String>>,
    bin_path: Option<String>,
    bin_arg: Option<Vec<String>>,
) -> Result<(), impl std::error::Error + Send + Sync> {
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
            command: Some("echo 'test'".to_string()),
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
