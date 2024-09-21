use crate::args::Args;
use log::{error, info, warn};
use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};
use std::{
    fs::metadata,
    io::{BufRead, BufReader},
    path::Path,
    process::{Child, Stdio},
    result::Result,
    sync::mpsc::channel,
    time::Duration,
};

pub async fn watch_dir(args: Args) -> notify::Result<()> {
    let (tx, rx) = channel();

    let mut watcher = recommended_watcher(move |res: Result<Event, notify::Error>| {
        tx.send(res).unwrap();
    })
    .unwrap();

    watcher
        .watch(args.dir.as_ref(), RecursiveMode::Recursive)
        .unwrap();

    info!("Waching directory: {:?}", args.dir);
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
                            .filter(|path| !is_ignored(path, &args.ignore))
                            .collect::<Vec<_>>();

                        if !paths.is_empty() {
                            info!("File content changed: {:?}", paths);

                            if let Some(ref mut child) = running_binary {
                                match child.kill() {
                                    Ok(_) => info!("Killed the running binary"),
                                    Err(e) => error!("Failed to kill binary: {:?}", e),
                                }
                            }

                            if let Some(bin_path) = &args.bin_path {
                                if remove_old_binary(bin_path) {
                                    if !binary_exists(bin_path) {
                                        match run_command(args.command.clone()).await {
                                            Ok(child) => {
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
                                            Err(e) => {
                                                error!("Failed to run command: {}", e)
                                            }
                                        }
                                    }

                                    running_binary =
                                        match restart_binary(bin_path, args.bin_arg.clone()) {
                                            Ok(child) => Some(child),
                                            Err(e) => {
                                                error!("Failed to restart binary: {:?}", e);
                                                None
                                            }
                                        }
                                }
                            } else {
                                match run_command(args.command.clone()).await {
                                    Ok(child) => {
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

    Ok(())
}

async fn run_command(cmd: String) -> Result<Child, Box<dyn std::error::Error>> {
    let output = tokio::task::spawn_blocking(move || {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap()
    })
    .await?;

    Ok(output)
}

fn is_ignored<P: AsRef<Path>>(path: P, ignored_patterns: &[String]) -> bool {
    let path_str = path.as_ref().to_str().unwrap_or("");

    for pattern in ignored_patterns {
        if path_str.ends_with(pattern) {
            return true;
        }
    }

    false
}

fn remove_old_binary(binary_path: &str) -> bool {
    match metadata(binary_path) {
        Ok(_) => {
            warn!("Remove old binary: {}", binary_path);
            match std::fs::remove_file(binary_path) {
                Ok(_) => {
                    warn!("Old binary removed");
                    true
                }
                Err(e) => {
                    error!("Failed to remove binary: {:?}", e);
                    false
                }
            }
        }
        Err(_) => {
            info!("No binary found to remove.");
            true
        }
    }
}

fn binary_exists(binary_path: &str) -> bool {
    metadata(binary_path).is_ok()
}

fn restart_binary(
    binary_path: &str,
    cmd_arg: Option<Vec<String>>,
) -> Result<Child, std::io::Error> {
    info!("Restarting binary: {}", binary_path);
    match cmd_arg {
        Some(args) => match std::process::Command::new(binary_path).args(args).spawn() {
            Ok(child) => {
                info!("Binary started: {}", child.id());
                Ok(child)
            }
            Err(e) => {
                error!("Failed to restart: {:?}", e);
                Err(e)
            }
        },
        _ => match std::process::Command::new(binary_path).spawn() {
            Ok(child) => {
                info!("Binary started: {}", child.id());
                Ok(child)
            }
            Err(e) => {
                error!("Failed to restart: {:?}", e);
                Err(e)
            }
        },
    }
}
