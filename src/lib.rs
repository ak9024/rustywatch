pub mod args;
pub mod config;
pub mod logger;
pub mod watch;

use args::Args;
use log::error;
use std::error::Error;

pub async fn run(args: Args) -> Result<(), Box<dyn Error>> {
    if let Err(err) = watch::watch(args).await {
        error!("Error watching directory: {:?}", err)
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_run() {
        // TODO: Implement test cases
    }
}
