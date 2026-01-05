use crate::config::schema::CommandType;
use crate::watch::reload::reload;
use log::info;
use std::collections::HashMap;
use std::process::Child;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReloadState {
    Idle,
    Reloading,
    PendingReload,
}

/// Controls non-blocking reload operations
pub struct ReloadController {
    state: Arc<Mutex<ReloadState>>,
    running_binary: Arc<Mutex<Option<Child>>>,
    cmd: CommandType,
    bin_path: Option<String>,
    bin_arg: Option<Vec<String>>,
    env_vars: HashMap<String, String>,
    dir: String,
}

impl ReloadController {
    pub fn new(
        cmd: CommandType,
        bin_path: Option<String>,
        bin_arg: Option<Vec<String>>,
        env_vars: HashMap<String, String>,
        dir: String,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(ReloadState::Idle)),
            running_binary: Arc::new(Mutex::new(None)),
            cmd,
            bin_path,
            bin_arg,
            env_vars,
            dir,
        }
    }

    /// Request a reload - non-blocking, returns immediately
    pub async fn request_reload(&self) {
        let mut state = self.state.lock().await;
        match *state {
            ReloadState::Idle => {
                *state = ReloadState::Reloading;
                drop(state);
                self.spawn_reload();
            }
            ReloadState::Reloading => {
                *state = ReloadState::PendingReload;
                info!("Reload already in progress, queuing next reload");
            }
            ReloadState::PendingReload => {
                // Already pending, no action needed
            }
        }
    }

    fn spawn_reload(&self) {
        let state = Arc::clone(&self.state);
        let running_binary = Arc::clone(&self.running_binary);
        let cmd = self.cmd.clone();
        let bin_path = self.bin_path.clone();
        let bin_arg = self.bin_arg.clone();
        let env_vars = self.env_vars.clone();
        let dir = self.dir.clone();

        tokio::spawn(async move {
            loop {
                // Perform the actual reload
                {
                    let mut binary = running_binary.lock().await;
                    reload(
                        &mut *binary,
                        &cmd,
                        bin_path.as_ref(),
                        bin_arg.as_ref(),
                        &env_vars,
                        &dir,
                    )
                    .await;
                }

                // Check if another reload was requested
                let mut current_state = state.lock().await;
                match *current_state {
                    ReloadState::PendingReload => {
                        *current_state = ReloadState::Reloading;
                        info!("Processing queued reload");
                        // Continue loop to perform another reload
                    }
                    _ => {
                        *current_state = ReloadState::Idle;
                        break;
                    }
                }
            }
        });
    }

    /// Perform initial reload (blocking for startup)
    pub async fn initial_reload(&self) {
        let mut binary = self.running_binary.lock().await;
        reload(
            &mut *binary,
            &self.cmd,
            self.bin_path.as_ref(),
            self.bin_arg.as_ref(),
            &self.env_vars,
            &self.dir,
        )
        .await;
    }

    /// Get current state
    pub async fn get_state(&self) -> ReloadState {
        *self.state.lock().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reload_controller_initial_state() {
        let controller = ReloadController::new(
            CommandType::Single("echo test".to_string()),
            None,
            None,
            HashMap::new(),
            ".".to_string(),
        );
        assert_eq!(controller.get_state().await, ReloadState::Idle);
    }

    #[tokio::test]
    async fn test_reload_controller_request_reload() {
        let controller = ReloadController::new(
            CommandType::Single("echo test".to_string()),
            None,
            None,
            HashMap::new(),
            ".".to_string(),
        );

        controller.request_reload().await;

        // State should be Reloading or back to Idle (if fast enough)
        let state = controller.get_state().await;
        assert!(state == ReloadState::Reloading || state == ReloadState::Idle);
    }
}
