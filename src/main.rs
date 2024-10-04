use clap::Parser;
use log::warn;
use rustywatch::{
    args::{self, Args},
    logger, run,
};
use std::path::Path;

#[tokio::main]
async fn main() {
    args::title();

    logger::setup_logging();

    let args = Args::parse();

    match Path::new(&args.config).exists() {
        true => run::config(args)
            .await
            .unwrap_or_else(|e: Box<dyn std::error::Error>| warn!("Error to execute: {}", e)),
        false => run::cli(args)
            .await
            .unwrap_or_else(|e: Box<dyn std::error::Error>| warn!("Error to execute: {}", e)),
    }
}
