use clap::Parser;
use env_logger::{self, Builder, Env};
use rustywatch::{args::Args, run};
use std::{env, process};

#[tokio::main]
async fn main() {
    setup_loging();
    let args = Args::parse();
    match run(args) {
        Ok(_) => process::exit(0),
        Err(e) => {
            eprintln!("{e}");
            process::exit(1)
        }
    }
}

fn setup_loging() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    Builder::from_env(Env::default().filter_or("RUST_LOG", "info")).init()
}
