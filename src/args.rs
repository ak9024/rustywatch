use clap::Parser;

const TITLE: &str = r#"
╭━━━╮╱╱╱╱╱╭╮╱╱╱╱╭╮╭╮╭╮╱╱╭╮╱╱╱╭╮
┃╭━╮┃╱╱╱╱╭╯╰╮╱╱╱┃┃┃┃┃┃╱╭╯╰╮╱╱┃┃
┃╰━╯┣╮╭┳━┻╮╭╋╮╱╭┫┃┃┃┃┣━┻╮╭╋━━┫╰━╮
┃╭╮╭┫┃┃┃━━┫┃┃┃╱┃┃╰╯╰╯┃╭╮┃┃┃╭━┫╭╮┃
┃┃┃╰┫╰╯┣━━┃╰┫╰━╯┣╮╭╮╭┫╭╮┃╰┫╰━┫┃┃┃
╰╯╰━┻━━┻━━┻━┻━╮╭╯╰╯╰╯╰╯╰┻━┻━━┻╯╰╯
╱╱╱╱╱╱╱╱╱╱╱╱╭━╯┃
╱╱╱╱╱╱╱╱╱╱╱╱╰━━╯"#;

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
    pub command: Option<String>,

    #[arg(short = 'i', long)]
    pub ignore: Option<Vec<String>>,

    #[arg(long)]
    pub bin_path: Option<String>,

    #[arg(short = 'a', long)]
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
            command: Some(String::from("test_command")),
            ignore: Some(vec![String::from(".git")]),
            bin_path: None,
            bin_arg: Some(vec![String::from("server")]),
            config: String::from("rustywatch.yaml"),
        };

        assert_eq!(args.dir.unwrap(), "/test/dir");
        assert_eq!(args.command.unwrap(), "test_command");
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
