use clap::Parser;
use rustywatch::{args::Args, run};
use std::process;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match run(args) {
        Ok(_) => process::exit(0),
        Err(e) => {
            eprintln!("{e}");
            process::exit(1)
        }
    }
}
