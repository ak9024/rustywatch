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

    match watcher.watch(dir.as_ref(), RecursiveMode::Recursive) {
        Ok(_) => {
            info!("Waching directory: {:?}", dir);
            info!("Please make any changes to starting");

            let mut running_binary: Option<Child> = None;
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
                                                    Ok(child) => print_into_shell(child),
                                                    Err(e) => {
                                                        error!("Failed to run command: {}", e)
                                                    }
                                                }
                                            }

                                            running_binary =
                                                match restart(bin_path, bin_arg.clone()) {
                                                    Ok(child) => Some(child),
                                                    Err(e) => {
                                                        error!("Failed to restart binary: {:?}", e);
                                                        None
                                                    }
                                                }
                                        }
                                    } else {
                                        match exec(cmd.clone()).await {
                                            Ok(child) => print_into_shell(child),
                                            Err(e) => {
                                                error!("Failed to run command: {}", e)
                                            }
                                        }
                                    }
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

fn print_into_shell(child: Child) {
    let stdout = child.stdout.unwrap();
    let stderr = child.stderr.unwrap();
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    for line in stdout_reader.lines() {
        println!("{}", line.unwrap());
    }

    for line in stderr_reader.lines() {
        eprintln!("{}", line.unwrap());
    }
}
