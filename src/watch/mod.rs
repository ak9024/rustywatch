use log::{error, info};
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

pub async fn watch_dir<P: AsRef<Path>>(
    path: P,
    command: String,
    ignored_patterns: Vec<String>,
) -> notify::Result<()> {
    let (tx, rx) = channel();

    let mut watcher = recommended_watcher(move |res: Result<Event, notify::Error>| {
        tx.send(res).unwrap();
    })?;

    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    info!("Waching directory: {:?}", path.as_ref());

    loop {
        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(event)) => {
                let filtered_paths: Vec<_> = event
                    .paths
                    .into_iter()
                    .filter(|path| !is_ignored(path, &ignored_patterns))
                    .collect();

                if !filtered_paths.is_empty() {
                    if let EventKind::Modify(modify_kind) = event.kind {
                        if matches!(modify_kind, notify::event::ModifyKind::Data(_)) {
                            for path in filtered_paths {
                                if let Ok(metadata) = metadata(&path) {
                                    if metadata.is_file() {
                                        match run_command(command.clone()).await {
                                            Ok(child) => {
                                                info!("File changed: {:?}", path);
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
