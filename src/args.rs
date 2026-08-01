use clap::{Parser, Subcommand};

const TITLE: &str = r#"
 ____            _       __        __    _       _
|  _ \ _   _ ___| |_ _   \ \      / /_ _| |_ ___| |__
| |_) | | | / __| __| | | \ \ /\ / / _` | __/ __| '_ \
|  _ <| |_| \__ \ |_| |_| |\ V  V / (_| | || (__| | | |
|_| \_\\__,_|___/\__|\__, | \_/\_/ \__,_|\__\___|_| |_|
                     |___/
"#;

pub fn title() {
    println!("{}", TITLE);
}

#[derive(Parser, Debug)]
#[command(
    version,
    author = clap::crate_authors!("\n"),
    about,
    args_conflicts_with_subcommands = true,
    help_template = "\
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading}
  {usage}

{all-args}{after-help}
",
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[command(flatten)]
    pub watch_args: WatchArgs,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new rustywatch.yaml configuration file
    Init(InitArgs),
}

#[derive(Debug, clap::Args)]
pub struct InitArgs {
    /// Output file path for the configuration
    #[arg(short = 'o', long = "output", default_value = "rustywatch.yaml")]
    pub output: String,

    /// Skip confirmation prompts and use defaults based on detected project type
    #[arg(long)]
    pub yes: bool,

    /// Force overwrite existing configuration file
    #[arg(short = 'f', long)]
    pub force: bool,
}

#[derive(Debug, clap::Args)]
pub struct WatchArgs {
    #[arg(short = 'd', long = "dir", default_value = ".")]
    pub dir: Option<String>,

    #[arg(short = 'c', long = "cmd")]
    pub command: Option<Vec<String>>,

    #[arg(short = 'i', long)]
    pub ignore: Option<Vec<String>>,

    #[arg(long)]
    pub bin_path: Option<String>,

    #[arg(long, allow_hyphen_values = true)]
    pub bin_arg: Option<Vec<String>>,

    #[arg(long = "cfg", default_value_t = String::from("rustywatch.yaml"))]
    pub config: String,

    #[arg(long = "monitor", help = "Show process monitoring dashboard")]
    pub monitor: bool,
}

/// Backward compatibility alias for WatchArgs
pub type Args = WatchArgs;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_args_creation() {
        let args = WatchArgs {
            dir: Some(String::from("/test/dir")),
            command: Some(vec![String::from("test_command")]),
            ignore: Some(vec![String::from(".git")]),
            bin_path: None,
            bin_arg: Some(vec![String::from("server")]),
            config: String::from("rustywatch.yaml"),
            monitor: false,
        };

        assert_eq!(args.dir.unwrap(), "/test/dir");
        assert_eq!(args.command.unwrap()[0], "test_command");
        assert_eq!(args.ignore.unwrap()[0], ".git");

        match args.bin_path {
            Some(cmd_bin) => assert_eq!(cmd_bin, ""),
            None => assert!(args.bin_path.is_none()),
        };

        if let Some(arg) = args.bin_arg {
            for a in arg {
                assert_eq!(a.as_str(), "server")
            }
        }

        assert_eq!(args.config, String::from("rustywatch.yaml"))
    }

    #[test]
    fn test_init_args_defaults() {
        let init_args = InitArgs {
            output: String::from("rustywatch.yaml"),
            yes: false,
            force: false,
        };

        assert_eq!(init_args.output, "rustywatch.yaml");
        assert!(!init_args.yes);
        assert!(!init_args.force);
    }
}
