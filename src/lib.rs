pub mod args;

use args::Args;
use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;
use std::{error::Error, result::Result};
use tokio::task;

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

    println!("Watching directory: {}", args.dir);

    while let Ok(res) = rx.recv() {
        match res {
            Ok(event) => {
                if matches!(event.kind, EventKind::Modify(_)) {
                    println!("File change detected: {:?}", event);

                    let cmd = args.command.clone();
                    task::spawn(async move {
                        if let Err(e) = Command::new("sh").arg("-c").arg(&cmd).status() {
                            eprintln!("Failed to run command: {}", e)
                        }
                    });
                }
            }
            Err(e) => println!("Error watching directory: {:?}", e),
        }
    }

    Ok(())
}
