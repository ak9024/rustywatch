use clap::Parser;
use futures::future::join_all;
use log::{error, info};
use rustywatch::{
    args::{self, Args},
    config::{self, CommandType},
    logger, run,
};
use std::{path::Path, process};
use validator::Validate;

#[tokio::main]
async fn main() {
    args::title();

    logger::setup_logging();

    let args = Args::parse();

    if !Path::new(&args.config).exists() {
        let dir = args.dir.unwrap_or_else(|| {
            info!("Please define your directory --dir <dir>");
            ".".to_string()
        });
        let cmd = CommandType::Single("echo 'File changed!'".to_string());
        let ignore = args.ignore;
        let bin_path = args.bin_path;
        let bin_arg = args.bin_arg;

        match run(dir, cmd, ignore, bin_path, bin_arg).await {
            Ok(_) => process::exit(0),
            Err(e) => {
                error!("{e}");
                process::exit(1)
            }
        }
    } else {
        match config::read_config(args.config) {
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

                    process::exit(0)
                }
                Err(e) => {
                    error!("Config validation failed: {}", e);
                    process::exit(1);
                }
            },

            Err(e) => {
                error!("Missing field workspaces at your config: {}", e)
            }
        };
    }
}
