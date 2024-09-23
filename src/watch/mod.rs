mod binary;
mod command;
mod filter;

use binary::{exists, remove, restart};
use command::exec;
use filter::is_ignored;
use log::{error, info};
use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};
use std::{
    io::{BufRead, BufReader},
    process::Child,
    result::Result,
    sync::mpsc::channel,
    time::Duration,
};

pub async fn watch(
    dir: String,
    cmd: String,
    ignore: Option<Vec<String>>,
    bin_path: Option<String>,
    bin_arg: Option<Vec<String>>,
) -> notify::Result<()> {
    let (tx, rx) = channel();

    let mut watcher = recommended_watcher(move |res: Result<Event, notify::Error>| {
        tx.send(res).unwrap();
    })
    .unwrap();

    let mut _ignore = ignore.unwrap_or_else(|| vec![".git".to_string()]);

    let mut running_binary: Option<Child> = None;

    // init for first starter
    reload(
        &mut running_binary,
        bin_path.clone(),
        cmd.clone(),
        bin_arg.clone(),
    )
    .await;

    match watcher.watch(dir.as_ref(), RecursiveMode::Recursive) {
        Ok(_) => {
            info!("Waching directory: {:?}", dir);
            loop {
                match rx.recv_timeout(Duration::from_secs(5)) {
                    Ok(Ok(event)) => {
                        if let EventKind::Modify(modify_kind) = event.kind {
                            if matches!(modify_kind, notify::event::ModifyKind::Data(_)) {
                                let paths = event
                                    .paths
                                    .iter()
                                    .filter(|path| !is_ignored(path, &_ignore))
                                    .collect::<Vec<_>>();

                                if !paths.is_empty() {
                                    info!("File changed: {:?}", paths);

                                    // second changes in loop
                                    reload(
                                        &mut running_binary,
                                        bin_path.as_ref().cloned(),
                                        cmd.clone(),
                                        bin_arg.clone(),
                                    )
                                    .await;
                                }
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        error!("Watch error: {:?}", e);
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        continue;
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        break;
                    }
                }
            }
        }
        Err(e) => {
            error!("Error to watching directory: {:?}", e.paths)
        }
    }

    Ok(())
}

async fn reload(
    running_binary: &mut Option<Child>,
    bin_path: Option<String>,
    cmd: String,
    bin_arg: Option<Vec<String>>,
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
                match exec(cmd.clone()).await {
                    Ok(child) => cmd_result(child),
                    Err(e) => {
                        error!("Failed to run command: {}", e)
                    }
                }
            }

            match restart(bin_path, bin_arg.clone()) {
                Ok(child) => *running_binary = Some(child),
                Err(e) => {
                    error!("Failed to restart binary: {:?}", e);
                    *running_binary = None
                }
            }
        }
    } else {
        match exec(cmd.clone()).await {
            Ok(child) => cmd_result(child),
            Err(e) => {
                error!("Failed to run command: {}", e)
            }
        }
    }
}

fn cmd_result(child: Child) {
    macro_rules! process_output {
        ($reader:expr, $print_fn:ident) => {
            for line in $reader.lines() {
                $print_fn!("{}", line.unwrap());
            }
        };
    }

    let stdout = child.stdout.unwrap();
    let stderr = child.stderr.unwrap();
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    process_output!(stdout_reader, println);
    process_output!(stderr_reader, eprintln);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_is_ignored() {
        let ignore_list = vec![".git".to_string(), "target".to_string()];
        assert!(is_ignored(&PathBuf::from(".git"), &ignore_list));
        assert!(is_ignored(&PathBuf::from("target"), &ignore_list));
        assert!(!is_ignored(&PathBuf::from("src"), &ignore_list));
    }

    #[tokio::test]
    async fn test_reload_without_bin_path() {
        let mut running_binary = None;
        let cmd = "echo 'Hello, World!'".to_string();

        reload(&mut running_binary, None, cmd.clone(), None).await;

        // Assert that running_binary is still None after reload
        assert!(running_binary.is_none());
    }

    #[tokio::test]
    async fn test_watch_timeout() {
        let dir = tempdir().unwrap();
        let cmd = "echo 'Test'".to_string();

        let handle = tokio::spawn(async move {
            watch(
                dir.path().to_str().unwrap().to_string(),
                cmd,
                None,
                None,
                None,
            )
            .await
        });

        // Cancel the task (simulating a timeout)
        handle.abort();

        let result = handle.await;
        assert!(result.is_err());
    }

    // Add more tests as needed
}
