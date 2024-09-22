pub mod args;
pub mod config;
pub mod logger;
pub mod watch;

use log::error;
use std::error::Error;

pub async fn run(
    dir: String,
    cmd: String,
    ignore: Option<Vec<String>>,
    bin_path: Option<String>,
    bin_arg: Option<Vec<String>>,
) -> Result<(), Box<dyn Error>> {
    if let Err(err) = watch::watch(dir, cmd, ignore, bin_path, bin_arg).await {
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
