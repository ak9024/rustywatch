use clap::Parser;
use rustywatch::{
    args::{self, Args},
    logger, run,
};
use std::process;

#[tokio::main]
async fn main() {
    args::title();
    logger::setup_logging();
    let args = Args::parse();
    match run(args).await {
        Ok(_) => process::exit(0),
        Err(e) => {
            eprintln!("{e}");
            process::exit(1)
        }
    }
}
