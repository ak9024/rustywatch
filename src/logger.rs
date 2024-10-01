use env_logger::{Builder, Env};
use std::env;

pub fn setup_logging() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    Builder::from_env(Env::default().filter_or("RUST_LOG", "info"))
        .format_timestamp(None)
        .format_indent(None)
        .format_target(false)
        .init()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_setup_logging() {
        env::remove_var("RUST_LOG");
        setup_logging();
        assert_eq!(env::var("RUST_LOG").unwrap(), "info");
    }
}
