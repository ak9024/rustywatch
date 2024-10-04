use crate::{
    config::entity::CommandType,
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
    match running_binary {
        Some(ref mut child) => match child.kill() {
            Ok(_) => info!("Killed the running binary"),
            Err(e) => error!("Failed to kill binary: {:?}", e),
        },
        None => (),
    }

    match bin_path {
        Some(bin_path) => {
            if remove(bin_path) {
                if !exists(bin_path) {
                    match cmd {
                        CommandType::Single(cmd) => match exec(cmd.clone()).await {
                            Ok(child) => buf_reader(child),
                            Err(e) => {
                                error!("Failed to run command: {}", e)
                            }
                        },
                        CommandType::Multiple(cmds) => {
                            for cmd in cmds {
                                match exec(cmd.clone()).await {
                                    Ok(child) => buf_reader(child),
                                    Err(e) => {
                                        error!("Failed to run command: {}", e)
                                    }
                                }
                            }
                        }
                    };
                }

                if cfg!(not(test)) {
                    match restart(bin_path, bin_arg) {
                        Ok(child) => *running_binary = Some(child),
                        Err(e) => {
                            error!("Failed to restart binary: {:?}", e);
                            *running_binary = None
                        }
                    }
                }
            }
        }
        None => match cmd {
            CommandType::Single(cmd) => match exec(cmd.clone()).await {
                Ok(child) => buf_reader(child),
                Err(e) => {
                    error!("Failed to run command: {}", e)
                }
            },
            CommandType::Multiple(cmds) => {
                for cmd in cmds {
                    match exec(cmd.clone()).await {
                        Ok(child) => buf_reader(child),
                        Err(e) => {
                            error!("Failed to run command: {}", e)
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
