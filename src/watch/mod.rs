mod binary;
mod command;
mod filter;

use crate::args::Args;
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

pub async fn watch(args: Args) -> notify::Result<()> {
    let (tx, rx) = channel();

    let mut watcher = recommended_watcher(move |res: Result<Event, notify::Error>| {
        tx.send(res).unwrap();
    })
    .unwrap();

    match watcher.watch(args.dir.as_ref(), RecursiveMode::Recursive) {
        Ok(_) => {
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
                                    info!("File changed: {:?}", paths);

                                    if let Some(ref mut child) = running_binary {
                                        match child.kill() {
                                            Ok(_) => info!("Killed the running binary"),
                                            Err(e) => error!("Failed to kill binary: {:?}", e),
                                        }
                                    }

                                    if let Some(bin_path) = &args.bin_path {
                                        if remove(bin_path) {
                                            if !exists(bin_path) {
                                                match exec(args.command.clone()).await {
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
                                                match restart(bin_path, args.bin_arg.clone()) {
                                                    Ok(child) => Some(child),
                                                    Err(e) => {
                                                        error!("Failed to restart binary: {:?}", e);
                                                        None
                                                    }
                                                }
                                        }
                                    } else {
                                        match exec(args.command.clone()).await {
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
        }
        Err(e) => {
            error!("Error to watching directory: {:?}", e.paths)
        }
    }

    Ok(())
}
