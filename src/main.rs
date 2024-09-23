use clap::Parser;
use futures::future::join_all;
use log::{error, info, warn};
use rustywatch::{
    args::{self, Args},
    config, logger, run,
};
use std::{path::Path, process};

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
        let cmd = args.command.unwrap_or_else(|| {
            warn!("Please define your command --cmd <your_command>");
            "echo 'file changed!'".to_string()
        });
        let ignore = args.ignore;
        let bin_path = args.bin_path;
        let bin_arg = args.bin_arg;

        match run(dir, cmd, ignore, bin_path, bin_arg, false).await {
            Ok(_) => process::exit(0),
            Err(e) => {
                error!("{e}");
                process::exit(1)
            }
        }
    } else {
        let config = config::read_config(args.config).unwrap();

        let tasks: Vec<_> = config
            .workspaces
            .into_iter()
            .map(|workspace| {
                tokio::spawn(async {
                    run(
                        workspace.dir,
                        workspace.cmd,
                        workspace.ignore,
                        workspace.bin_path,
                        workspace.bin_arg,
                        false,
                    )
                    .await
                })
            })
            .collect();

        let results = join_all(tasks).await;

        for result in results {
            match result {
                Ok(_) => continue,
                Err(e) => {
                    error!("Task panicked: {}", e);
                    process::exit(1);
                }
            }
        }
        process::exit(0)
    }
}
