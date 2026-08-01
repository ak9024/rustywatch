use crate::{
    config::schema::{CommandType, Workspace},
    watch::{binary, command},
};
use binary::{exists, remove, restart};
use command::{buf_reader_async, exec};
use futures::future::join_all;
use log::{error, info};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::Child;

/// Execute commands based on CommandType
/// Commands are executed in the specified working directory
async fn execute_commands(cmd: &CommandType, env_vars: &HashMap<String, String>, work_dir: &str) {
    let tasks: Vec<_> = cmd
        .iter()
        .filter(|c| !c.trim().is_empty())
        .map(|c| {
            let c = c.to_string();
            let env_vars = env_vars.clone();
            let work_dir = work_dir.to_string();
            async move {
                match exec(&c, &env_vars, &work_dir).await {
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

    // `CommandType::Multiple` runs every command in parallel, not in sequence.
    join_all(tasks).await;
}

/// Resolves `bin_path` to an absolute path.
///
/// Absolute paths are used as-is; relative paths resolve against
/// `cwd + work_dir + bin_path`, i.e. relative to the workspace directory rather
/// than to the process working directory.
pub fn resolve_bin_path(bin_path: &str, work_dir: &str) -> String {
    if Path::new(bin_path).is_absolute() {
        return bin_path.to_string();
    }

    env::current_dir()
        .map(|cwd| {
            cwd.join(work_dir)
                .join(bin_path)
                .to_string_lossy()
                .to_string()
        })
        .unwrap_or_else(|_| {
            Path::new(work_dir)
                .join(bin_path)
                .to_string_lossy()
                .to_string()
        })
}

/// Kills any previously spawned binary, runs the workspace command(s), then
/// respawns the binary at `bin_path` if one is configured.
///
/// `running_binary` is updated in place: it holds the newly spawned child, or
/// `None` when the workspace has no binary or the build did not produce one.
pub async fn reload(
    running_binary: &mut Option<Child>,
    workspace: &Workspace,
    env_vars: &HashMap<String, String>,
) {
    let work_dir = workspace.dir.as_str();

    // Kill old binary if running
    if let Some(ref mut child) = running_binary {
        match child.kill() {
            Ok(_) => info!("Restarting..."),
            Err(e) => error!("Failed to restart binary: {:?}", e.to_string()),
        }
    }

    let Some(bin_path) = workspace.bin_path.as_deref() else {
        execute_commands(&workspace.cmd, env_vars, work_dir).await;
        return;
    };

    let absolute_bin_path = resolve_bin_path(bin_path, work_dir);

    if !remove(&absolute_bin_path) {
        error!("Failed to remove stale binary: {}", absolute_bin_path);
        return;
    }

    if !exists(&absolute_bin_path) {
        execute_commands(&workspace.cmd, env_vars, work_dir).await;
    }

    // Prevent restart in test environment
    if cfg!(test) {
        return;
    }

    // Skip restart if binary doesn't exist after build
    if !exists(&absolute_bin_path) {
        *running_binary = None;
        return;
    }

    // Restart the binary using absolute path
    match restart(
        &absolute_bin_path,
        workspace.bin_arg.as_ref(),
        env_vars,
        work_dir,
    ) {
        Ok(child) => *running_binary = Some(child),
        Err(e) => {
            error!("Failed to restart binary: {:?}", e.to_string());
            error!("Please check your <bin_path>: {}", absolute_bin_path);
            *running_binary = None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[tokio::test]
    async fn test_reload_with_no_binary() {
        let mut running_binary = None;
        let workspace = Workspace::new(".").cmd("echo test");
        let env_vars = HashMap::new();

        reload(&mut running_binary, &workspace, &env_vars).await;
        assert!(running_binary.is_none());
    }

    #[tokio::test]
    async fn test_reload_with_binary() {
        let mut running_binary = Some(Command::new("sleep").arg("1000").spawn().unwrap());
        let workspace = Workspace::new(".")
            .cmd("echo test")
            .bin_path("test_binary")
            .bin_arg(["arg1", "arg2"]);
        let env_vars = HashMap::new();

        reload(&mut running_binary, &workspace, &env_vars).await;
        assert!(running_binary.is_some());
    }

    #[tokio::test]
    async fn test_reload_with_multiple_commands() {
        let mut running_binary = None;
        let workspace = Workspace::new(".").cmds(["echo first", "echo second", "echo third"]);
        let env_vars = HashMap::new();

        reload(&mut running_binary, &workspace, &env_vars).await;
        assert!(running_binary.is_none());
    }

    #[tokio::test]
    async fn test_reload_skips_blank_commands() {
        let mut running_binary = None;
        let workspace = Workspace::new(".");
        let env_vars = HashMap::new();

        reload(&mut running_binary, &workspace, &env_vars).await;
        assert!(running_binary.is_none());
    }

    #[test]
    fn test_resolve_bin_path_keeps_absolute() {
        assert_eq!(resolve_bin_path("/usr/bin/echo", "api"), "/usr/bin/echo");
    }

    #[test]
    fn test_resolve_bin_path_joins_workspace_dir() {
        let resolved = resolve_bin_path("target/debug/api", "api");
        let expected = env::current_dir()
            .unwrap()
            .join("api")
            .join("target/debug/api");

        assert_eq!(resolved, expected.to_string_lossy());
    }
}
