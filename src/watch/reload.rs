use crate::{
    config::CommandType,
    watch::{binary, command, command::cmd_result},
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
    if let Some(ref mut child) = running_binary {
        match child.kill() {
            Ok(_) => info!("Killed the running binary"),
            Err(e) => error!("Failed to kill binary: {:?}", e),
        }
    }

    if let Some(bin_path) = &bin_path {
        if remove(bin_path) {
            if !exists(bin_path) {
                match cmd {
                    CommandType::Single(cmd) => match exec(cmd.clone()).await {
                        Ok(child) => cmd_result(child),
                        Err(e) => {
                            error!("Failed to run command: {}", e)
                        }
                    },
                    CommandType::Multiple(cmds) => {
                        for cmd in cmds {
                            match exec(cmd.clone()).await {
                                Ok(child) => cmd_result(child),
                                Err(e) => {
                                    error!("Failed to run command: {}", e)
                                }
                            }
                        }
                    }
                };
            }

            match restart(bin_path, bin_arg) {
                Ok(child) => *running_binary = Some(child),
                Err(e) => {
                    error!("Failed to restart binary: {:?}", e);
                    *running_binary = None
                }
            }
        }
    } else {
        match cmd {
            CommandType::Single(cmd) => match exec(cmd.clone()).await {
                Ok(child) => cmd_result(child),
                Err(e) => {
                    error!("Failed to run command: {}", e)
                }
            },
            CommandType::Multiple(cmds) => {
                for cmd in cmds {
                    match exec(cmd.clone()).await {
                        Ok(child) => cmd_result(child),
                        Err(e) => {
                            error!("Failed to run command: {}", e)
                        }
                    }
                }
            }
        }
    }
}
