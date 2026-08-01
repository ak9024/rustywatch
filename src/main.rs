use clap::Parser;
use log::warn;
use rustywatch::{
    args::{self, Cli, Commands},
    init, logger, monitor, run,
};
use std::path::Path;

#[tokio::main]
async fn main() {
    args::title();

    logger::setup_logging();

    let cli = Cli::parse();

    // Handle subcommands
    match cli.command {
        Some(Commands::Init(init_args)) => {
            if let Err(e) = init::run(init_args) {
                warn!("Error during initialization: {}", e);
                std::process::exit(1);
            }
        }
        None => {
            // No subcommand - use watch args (backward compatible)
            let args = cli.watch_args;

            // Run the process monitor if the monitor flag is set
            if args.monitor {
                if let Err(e) = monitor::run(args.config.clone()) {
                    warn!("Error running process monitor: {}", e);
                }
                return;
            }

            // Otherwise run the file watcher. The config file wins over CLI
            // flags when it exists.
            let result = match Path::new(&args.config).exists() {
                true => run::config(args).await,
                false => run::cli(args).await,
            };

            if let Err(e) = result {
                warn!("Error to execute: {}", e);
                std::process::exit(1);
            }
        }
    }
}
