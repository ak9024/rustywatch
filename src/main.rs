use clap::Parser;
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

    println!("{args:?}");

    match args.config {
        Some(cfg) => {
            println!("{cfg:?}");
            let config = config::read_config(cfg).unwrap();
            println!("{:#?}", config);

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
            let dir = args.dir.unwrap_or_else(|| ".".to_string());
            let cmd = args
                .command
                .unwrap_or_else(|| "echo 'file changed!'".to_string());
            let ignore = args.ignore.unwrap_or_else(|| vec![".git".to_string()]);
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
