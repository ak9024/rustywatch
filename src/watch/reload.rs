use crate::{
    config::schema::CommandType,
    watch::{binary, command, command::buf_reader},
};
use binary::{exists, remove, restart};
use command::exec;
use log::{error, info};
use std::process::Child;

pub async fn reload(
    running_binary: &mut Option<Child>,
    cmd: &CommandType,
    bin_path: Option<&String>,
    bin_arg: Option<&Vec<String>>,
) {
    // @NOTE
    // restart with killed old binary.
    match running_binary {
        Some(ref mut child) => match child.kill() {
            Ok(_) => info!("Restarting..."),
            Err(e) => error!("Failed to restart binary: {:?}", e.to_string()),
        },
        None => (),
    }

    // @NOTE
    // execute bin_path with option:
    // if not exists execute.
    // then any skip, just execute command.
    match bin_path {
        Some(bin_path) => {
            if remove(bin_path) {
                if !exists(bin_path) {
                    match cmd {
                        CommandType::Single(cmd) => match exec(cmd.clone()).await {
                            Ok(child) => buf_reader(child),
                            Err(e) => {
                                error!("Failed to run command: {}", e.to_string())
                            }
                        },
                        CommandType::Multiple(cmds) => {
                            for cmd in cmds {
                                match exec(cmd.clone()).await {
                                    Ok(child) => buf_reader(child),
                                    Err(e) => {
                                        error!("Failed to run command: {}", e.to_string())
                                    }
                                }
                            }
                        }
                    };
                }

                // @NOTE
                // prevent restart in test environment
                if cfg!(test) {
                    return;
                }

                // @NOTE
                // restart the binary
                match restart(bin_path, bin_arg) {
                    Ok(child) => *running_binary = Some(child),
                    Err(e) => {
                        error!("Failed to restart binary: {:?}", e.to_string());
                        error!("Please check your <bin_path>: {}", bin_path);
                        *running_binary = None
                    }
                }
            }
        }

        // @NOTE
        // if bin_path not defined, just execute command.
        None => match cmd {
            CommandType::Single(cmd) => match exec(cmd.clone()).await {
                Ok(child) => buf_reader(child),
                Err(e) => {
                    error!("Failed to run command: {}", e.to_string())
                }
            },
            CommandType::Multiple(cmds) => {
                for cmd in cmds {
                    match exec(cmd.clone()).await {
                        Ok(child) => buf_reader(child),
                        Err(e) => {
                            error!("Failed to run command: {}", e.to_string())
                        }
                    }
                }
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[tokio::test]
    async fn test_reload_with_no_binary() {
        let mut running_binary = None;
        let cmd = CommandType::Single("echo test".to_string());
        let bin_path = None;
        let bin_arg = None;

        reload(&mut running_binary, &cmd, bin_path, bin_arg).await;
        assert!(running_binary.is_none());
    }

    #[tokio::test]
    async fn test_reload_with_binary() {
        let mut running_binary = Some(Command::new("sleep").arg("1000").spawn().unwrap());
        let cmd = CommandType::Single("echo test".to_string());
        let bin_path = Some("test_binary".to_string());
        let bin_arg = Some(vec!["arg1".to_string(), "arg2".to_string()]);

        reload(
            &mut running_binary,
            &cmd,
            bin_path.as_ref(),
            bin_arg.as_ref(),
        )
        .await;
        assert!(running_binary.is_some());
    }
}
