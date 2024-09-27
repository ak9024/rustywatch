use log::{error, info, warn};
use std::{fs::metadata, process::Child};

pub fn remove(binary_path: &str) -> bool {
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

pub fn exists(binary_path: &str) -> bool {
    metadata(binary_path).is_ok()
}

pub fn restart(binary_path: &str, cmd_arg: Option<&Vec<String>>) -> Result<Child, std::io::Error> {
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
