pub mod args;
pub mod logger;

use args::Args;
use core::str;
use log::{error, info};
use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};
use std::{error::Error, path::Path, process::Output, result::Result, str, sync::mpsc::channel};
use tokio::{process::Command as TokioCommand, task};

pub fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = channel();

    let mut watcher = recommended_watcher(move |res: Result<Event, notify::Error>| {
        tx.send(res).unwrap();
    })
    .expect("Failed to create watcher");

    let directory = Path::new(&args.dir);

    watcher
        .watch(directory, RecursiveMode::Recursive)
        .expect("Failed to watch directory");

    info!("Watching directory: {}", args.dir);

    while let Ok(res) = rx.recv() {
        match res {
            Ok(event) => {
                if matches!(event.kind, EventKind::Modify(_)) {
                    info!("File change detected: {:?}", event.paths);

                    let cmd = args.command.clone();
                    task::spawn(async move {
                        info!("Executing command: {}", cmd);
                        match run_command(&cmd).await {
                            Ok(output) => {
                                if output.status.success() {
                                    info!("{}", str::from_utf8(&output.stdout).unwrap())
                                } else {
                                    error!("Command failed with status: {}", output.status);
                                    error!("{}", str::from_utf8(&output.stderr).unwrap());
                                }
                            }
                            Err(e) => error!("Failed to run command: {}", e),
                        }
                    });
                }
            }
            Err(e) => error!("Error watching directory: {:?}", e),
        }
    }

    Ok(())
}

async fn run_command(cmd: &str) -> Result<Output, Box<dyn Error + Send + Sync>> {
    let output = if cfg!(target_os = "windows") {
        TokioCommand::new("cmd").arg("/C").arg(cmd).output().await?
    } else {
        TokioCommand::new("sh").arg("-c").arg(cmd).output().await?
    };

    Ok(output)
}
