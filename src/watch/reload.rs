use crate::{
    config::schema::CommandType,
    watch::{binary, command},
};
use binary::{exists, remove, restart};
use command::{buf_reader_async, exec};
use futures::future::join_all;
use log::{error, info};
use std::collections::HashMap;
use std::process::Child;

/// Execute commands based on CommandType
async fn execute_commands(cmd: &CommandType, env_vars: &HashMap<String, String>) {
    match cmd {
        CommandType::Single(c) => {
            match exec(c, env_vars).await {
                Ok(child) => {
                    if let Err(e) = buf_reader_async(child).await {
                        error!("Failed to read command output: {}", e);
                    }
                }
                Err(e) => error!("Failed to run command: {}", e),
            }
        }
        CommandType::Multiple(cmds) => {
            // Execute all commands in parallel
            let tasks: Vec<_> = cmds
                .iter()
                .map(|c| {
                    let env_vars = env_vars.clone();
                    async move {
                        match exec(c, &env_vars).await {
                            Ok(child) => {
                                if let Err(e) = buf_reader_async(child).await {
                                    error!("Failed to read command output: {}", e);
                                }
                            }
                            Err(e) => error!("Failed to run command: {}", e),
                        }
                    }
                })
                .collect();

            join_all(tasks).await;
        }
    }
}

pub async fn reload(
    running_binary: &mut Option<Child>,
    cmd: &CommandType,
    bin_path: Option<&String>,
    bin_arg: Option<&Vec<String>>,
    env_vars: &HashMap<String, String>,
) {
    // Kill old binary if running
    if let Some(ref mut child) = running_binary {
        match child.kill() {
            Ok(_) => info!("Restarting..."),
            Err(e) => error!("Failed to restart binary: {:?}", e.to_string()),
        }
    }

    match bin_path {
        Some(bin_path) => {
            if remove(bin_path) {
                if !exists(bin_path) {
                    execute_commands(cmd, env_vars).await;
                }

                // Prevent restart in test environment
                if cfg!(test) {
                    return;
                }

                // Restart the binary
                match restart(bin_path, bin_arg, env_vars) {
                    Ok(child) => *running_binary = Some(child),
                    Err(e) => {
                        error!("Failed to restart binary: {:?}", e.to_string());
                        error!("Please check your <bin_path>: {}", bin_path);
                        *running_binary = None
                    }
                }
            }
        }
        None => execute_commands(cmd, env_vars).await,
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
        let env_vars = HashMap::new();

        reload(&mut running_binary, &cmd, bin_path, bin_arg, &env_vars).await;
        assert!(running_binary.is_none());
    }

    #[tokio::test]
    async fn test_reload_with_binary() {
        let mut running_binary = Some(Command::new("sleep").arg("1000").spawn().unwrap());
        let cmd = CommandType::Single("echo test".to_string());
        let bin_path = Some("test_binary".to_string());
        let bin_arg = Some(vec!["arg1".to_string(), "arg2".to_string()]);
        let env_vars = HashMap::new();

        reload(
            &mut running_binary,
            &cmd,
            bin_path.as_ref(),
            bin_arg.as_ref(),
            &env_vars,
        )
        .await;
        assert!(running_binary.is_some());
    }

    #[tokio::test]
    async fn test_reload_with_multiple_commands() {
        let mut running_binary = None;
        let cmd = CommandType::Multiple(vec![
            "echo first".to_string(),
            "echo second".to_string(),
            "echo third".to_string(),
        ]);
        let bin_path = None;
        let bin_arg = None;
        let env_vars = HashMap::new();

        reload(&mut running_binary, &cmd, bin_path, bin_arg, &env_vars).await;
        assert!(running_binary.is_none());
    }
}
