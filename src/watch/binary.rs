use log::{error, info, warn};
use std::{
    fs::metadata,
    process::{Child, Command},
};

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
        Some(args) => match Command::new(binary_path).args(args).spawn() {
            Ok(child) => {
                info!("Binary started: {}", child.id());
                Ok(child)
            }
            Err(e) => {
                error!("Failed to restart: {:?}", e);
                Err(e)
            }
        },
        None => match Command::new(binary_path).spawn() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_remove() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_binary");
        File::create(&file_path).unwrap();

        assert!(remove(file_path.to_str().unwrap()));
        assert!(!exists(file_path.to_str().unwrap()));
    }

    #[test]
    fn test_exists() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_binary");

        assert!(!exists(file_path.to_str().unwrap()));
        File::create(&file_path).unwrap();
        assert!(exists(file_path.to_str().unwrap()));
    }

    #[test]
    fn test_restart() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_binary");
        File::create(&file_path)
            .unwrap()
            .write_all(b"#!/bin/sh\necho 'Test'")
            .unwrap();

        std::fs::set_permissions(
            &file_path,
            std::os::unix::fs::PermissionsExt::from_mode(0o755),
        )
        .unwrap();

        let result = restart(file_path.to_str().unwrap(), None);
        assert!(result.is_ok());

        let child = result.unwrap();
        assert!(child.id() > 0);
    }
}
