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
        let cmd = CommandType::Single(args.command.unwrap().to_string());
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
        match config::read_config(&args.config) {
            Ok(config) => {
                if let Err(e) = config.validate() {
                    error!("Config validation failed: {}", e);
                    process::exit(1);
                }

                let tasks = config.workspaces.into_iter().map(|workspace| {
                    tokio::spawn(async move {
                        if let Err(e) = run(
                            workspace.dir,
                            workspace.cmd,
                            workspace.ignore,
                            workspace.bin_path,
                            workspace.bin_arg,
                        )
                        .await
                        {
                            error!("Task failed: {}", e);
                        }
                    })
                });

                let results = join_all(tasks).await;

                for result in results {
                    if let Err(e) = result {
                        error!("Task panicked: {}", e);
                        process::exit(1);
                    }
                }

                process::exit(0)
            }
            Err(e) => {
                error!("Failed to read config: {}", e);
                process::exit(1);
            }
        }
    }
}
