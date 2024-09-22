use clap::Parser;
use log::{info, warn};
use rustywatch::{
    args::{self, Args},
    config, logger, run,
};
use std::process;

#[tokio::main]
async fn main() {
    args::title();

    logger::setup_logging();

    let args = Args::parse();

    match args.config {
        Some(cfg) => {
            let config = config::read_config(cfg).unwrap();
            match run(
                config.dir,
                config.cmd,
                config.ignore,
                config.bin_path,
                config.bin_arg,
            )
            .await
            {
                Ok(_) => process::exit(0),
                Err(e) => {
                    eprintln!("{e}");
                    process::exit(1)
                }
            }
        }
        None => {
            let dir = args.dir.unwrap_or_else(|| {
                info!("Plesae define your directory --dir <dir>");
                ".".to_string()
            });
            let cmd = args.command.unwrap_or_else(|| {
                warn!("Please define your command --cmd <your_command>");
                "echo 'file changed!'".to_string()
            });
            let ignore = args.ignore;
            let bin_path = args.bin_path;
            let bin_arg = args.bin_arg;

            match run(dir, cmd, ignore, bin_path, bin_arg).await {
                Ok(_) => process::exit(0),
                Err(e) => {
                    eprintln!("{e}");
                    process::exit(1)
                }
            }
        }
    }
}
