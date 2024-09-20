use log::{error, info};
use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};
use std::{
    error::Error, fs::metadata, path::Path, process::Output, result::Result, str,
    sync::mpsc::channel,
};

pub async fn watch_dir<P: AsRef<Path>>(path: P, command: String) -> notify::Result<()> {
    let (tx, rx) = channel();

    let mut watcher = recommended_watcher(move |res: Result<Event, notify::Error>| {
        tx.send(res).unwrap();
    })?;

    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    loop {
        match rx.recv() {
            Ok(Ok(event)) => match event.kind {
                EventKind::Modify(notify::event::ModifyKind::Data(_)) => {
                    for path in event.paths {
                        if let Ok(metadata) = metadata(&path) {
                            if metadata.is_file() {
                                match run_command(&command).await {
                                    Ok(output) => {
                                        if output.status.success() {
                                            info!("{}", str::from_utf8(&output.stdout).unwrap())
                                        } else {
                                            error!("{}", str::from_utf8(&output.stderr).unwrap())
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
                EventKind::Create(_) => {}
                _ => {}
            },
            Ok(Err(e)) => {
                error!("Error receiving event: {:?}", e);
            }
            Err(e) => {
                error!("Error receiving event: {:?}", e);
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
