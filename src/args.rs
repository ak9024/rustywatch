use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short = 'd', long = "dir", default_value = ".")]
    pub dir: String,

    #[arg(short = 'c', long = "cmd")]
    pub command: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_creation() {
        let args = Args {
            dir: String::from("/test/dir"),
            command: String::from("test_command"),
        };
        assert_eq!(args.dir, "/test/dir");
        assert_eq!(args.command, "test_command");
    }
}
