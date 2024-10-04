use clap::Parser;

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
#[clap(
    version,
    author = clap::crate_authors!("\n"),
    about,
    rename_all_env = "screaming-snake",
    help_template = "\
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading}
  {usage}

{all-args}{after-help}
",
)]
pub struct Args {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_creation() {
        let args = Args {
            dir: Some(String::from("/test/dir")),
            command: Some(vec![String::from("test_command")]),
            ignore: Some(vec![String::from(".git")]),
            bin_path: None,
            bin_arg: Some(vec![String::from("server")]),
            config: String::from("rustywatch.yaml"),
        };

        assert_eq!(args.dir.unwrap(), "/test/dir");
        assert_eq!(args.command.unwrap()[0], "test_command");
        assert_eq!(args.ignore.unwrap()[0], ".git");

        match args.bin_path {
            Some(cmd_bin) => assert_eq!(cmd_bin, ""),
            None => assert_eq!(args.bin_path.is_none(), true),
        };

        match args.bin_arg {
            Some(arg) => {
                for a in arg {
                    assert_eq!(a.as_str(), "server")
                }
            }
            None => {}
        }

        assert_eq!(args.config, String::from("rustywatch.yaml"))
    }
}
