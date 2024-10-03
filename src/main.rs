use clap::Parser;
use rustywatch::{
    args::{self, Args},
    logger,
    run::{run_with_config, run_without_config},
};
use std::path::Path;

#[tokio::main]
async fn main() {
    args::title();

    logger::setup_logging();

    let args = Args::parse();

    match Path::new(&args.config).exists() {
        true => run_with_config(args).await.unwrap(),
        false => run_without_config(args).await.unwrap(),
    }
}
