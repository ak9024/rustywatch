pub mod args;
pub mod logger;
pub mod watch;

use args::Args;
use log::error;
use std::error::Error;

pub async fn run(args: Args) -> Result<(), Box<dyn Error>> {
    if let Err(err) = watch::watch_dir(args.dir, args.command).await {
        error!("Error watching directory: {:?}", err)
    }

    Ok(())
}
