use log::{error, info};
use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};
use std::{
    error::Error, fs::metadata, path::Path, process::Output, result::Result, str,
    sync::mpsc::channel, time::Duration,
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

    info!("Waching dir: {:?}", path.as_ref());

    loop {
        match rx.recv_timeout(Duration::from_secs(10)) {
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
                                        match run_command(&command).await {
                                            Ok(output) => {
                                                if output.status.success() {
                                                    info!(
                                                        "{}",
                                                        str::from_utf8(&output.stdout).unwrap()
                                                    )
                                                } else {
                                                    error!(
                                                        "{}",
                                                        str::from_utf8(&output.stderr).unwrap()
                                                    )
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

async fn run_command(cmd: &str) -> Result<Output, Box<dyn Error + Send + Sync>> {
    let output = if cfg!(target_os = "windows") {
        tokio::process::Command::new("cmd")
            .arg("/C")
            .arg(cmd)
            .output()
            .await?
    } else {
        tokio::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .await?
    };

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
