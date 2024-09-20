use env_logger::{Builder, Env};
use std::env;

pub fn setup_logging() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    Builder::from_env(Env::default().filter_or("RUST_LOG", "info")).init()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_setup_logging() {
        // Clear the RUST_LOG environment variable before the test
        env::remove_var("RUST_LOG");

        // Call the setup_logging function
        setup_logging();

        // Check if RUST_LOG is set to "info"
        assert_eq!(env::var("RUST_LOG").unwrap(), "info");
    }
}
