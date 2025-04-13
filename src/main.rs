use clap::Parser;
use log::warn;
use rustywatch::{
    args::{self, Args},
    logger, monitor, run,
};
use std::path::Path;

#[tokio::main]
async fn main() {
    args::title();

    logger::setup_logging();

    let args = Args::parse();

    // Run the process monitor if the monitor flag is set
    if args.monitor {
        if let Err(e) = monitor::run() {
            warn!("Error running process monitor: {}", e);
        }
        return;
    }

    // Otherwise run the file watcher
    match Path::new(&args.config).exists() {
        true => run::config(args)
            .await
            .unwrap_or_else(|e| warn!("Error to execute: {}", e.to_string())),
        false => run::cli(args)
            .await
            .unwrap_or_else(|e| warn!("Error to execute: {}", e.to_string())),
    }
}
