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
const VERSION: &str = "v0.2.1";

pub fn title() {
    println!("{}", TITLE);
    println!("version: {}", VERSION);
    println!("\n");
}

#[derive(Parser, Debug)]
#[command(version = VERSION, about, long_about = None)]
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
