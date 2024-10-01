use crate::{
    config::CommandType,
    watch::{filter::is_ignored, reload::reload},
};

use log::{error, info};
use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};
use std::{
    process::Child,
    result::Result,
    sync::mpsc::{channel, RecvTimeoutError},
    time::Duration,
};

pub async fn watcher(
    dir: String,
    cmd: CommandType,
    ignore: Option<Vec<String>>,
    bin_path: Option<String>,
    bin_arg: Option<Vec<String>>,
) -> notify::Result<()> {
    let (tx, rx) = channel();

    let mut watcher = recommended_watcher(move |res: Result<Event, notify::Error>| {
        tx.send(res).unwrap();
    })
    .unwrap();

    let ignore = ignore.unwrap_or_else(|| vec![".git".to_string()]);

    let mut running_binary: Option<Child> = None;

    reload(
        &mut running_binary,
        &cmd,
        bin_path.as_ref(),
        bin_arg.as_ref(),
    )
    .await;

    // @NOTE
    // just skip in mode testing to prevent blocking
    if cfg!(not(test)) {
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
                                        .filter(|path| !is_ignored(path, &ignore))
                                        .collect::<Vec<_>>();

                                    if !paths.is_empty() {
                                        info!("File changed: {:?}", paths);

                                        reload(
                                            &mut running_binary,
                                            &cmd,
                                            bin_path.as_ref(),
                                            bin_arg.as_ref(),
                                        )
                                        .await;
                                    }
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            error!("Watch error: {:?}", e);
                        }
                        Err(RecvTimeoutError::Timeout) => {
                            continue;
                        }
                        Err(RecvTimeoutError::Disconnected) => {
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                error!("Error to watching directory: {:?}", e.paths)
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::write;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_watch() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path().to_str().unwrap().to_string();

        write(temp_dir.path().join("test.txt"), "initial content").unwrap();

        let cmd = CommandType::Single("echo".to_string());
        let ignore = Some(vec![".git".to_string()]);

        let watch_task = tokio::spawn(async move {
            watcher(dir_path, cmd, ignore, None, None).await.unwrap();
        });

        write(temp_dir.path().join("test.txt"), "modified content").unwrap();

        watch_task.abort();
    }
}
