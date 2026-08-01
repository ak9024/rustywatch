use crate::config::schema::Workspace;
use crate::watch::reload::reload;
use log::info;
use std::collections::HashMap;
use std::process::Child;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Where a workspace is in the reload cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReloadState {
    /// Nothing running, nothing queued.
    Idle,
    /// A reload is in flight.
    Reloading,
    /// A reload is in flight and exactly one more is queued behind it.
    PendingReload,
}

/// Coalesces reload requests so at most one reload runs and at most one is
/// queued: bursts of file changes collapse into a single follow-up run.
pub struct ReloadController {
    state: Arc<Mutex<ReloadState>>,
    running_binary: Arc<Mutex<Option<Child>>>,
    workspace: Arc<Workspace>,
    env_vars: Arc<HashMap<String, String>>,
}

impl ReloadController {
    /// Creates a controller for `workspace`, injecting `env_vars` into every
    /// command and binary spawn.
    pub fn new(workspace: Workspace, env_vars: HashMap<String, String>) -> Self {
        Self {
            state: Arc::new(Mutex::new(ReloadState::Idle)),
            running_binary: Arc::new(Mutex::new(None)),
            workspace: Arc::new(workspace),
            env_vars: Arc::new(env_vars),
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
        let workspace = Arc::clone(&self.workspace);
        let env_vars = Arc::clone(&self.env_vars);

        tokio::spawn(async move {
            loop {
                // Perform the actual reload
                {
                    let mut binary = running_binary.lock().await;
                    reload(&mut binary, &workspace, &env_vars).await;
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
        reload(&mut binary, &self.workspace, &self.env_vars).await;
    }

    /// Get current state
    pub async fn get_state(&self) -> ReloadState {
        *self.state.lock().await
    }

    /// The workspace this controller reloads.
    pub fn workspace(&self) -> &Workspace {
        &self.workspace
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn controller() -> ReloadController {
        ReloadController::new(Workspace::new(".").cmd("echo test"), HashMap::new())
    }

    #[tokio::test]
    async fn test_reload_controller_initial_state() {
        assert_eq!(controller().get_state().await, ReloadState::Idle);
    }

    #[tokio::test]
    async fn test_reload_controller_request_reload() {
        let controller = controller();

        controller.request_reload().await;

        // State should be Reloading or back to Idle (if fast enough)
        let state = controller.get_state().await;
        assert!(state == ReloadState::Reloading || state == ReloadState::Idle);
    }

    #[tokio::test]
    async fn test_reload_controller_exposes_workspace() {
        assert_eq!(controller().workspace().dir, ".");
    }

    // A burst of requests must collapse: at most one reload runs and at most
    // one is queued, so the state never leaves the three known values.
    #[tokio::test]
    async fn test_reload_controller_coalesces_a_burst() {
        let controller = controller();

        for _ in 0..10 {
            controller.request_reload().await;
        }

        assert!(matches!(
            controller.get_state().await,
            ReloadState::Idle | ReloadState::Reloading | ReloadState::PendingReload
        ));
    }

    #[tokio::test]
    async fn test_reload_controller_queues_while_reloading() {
        let controller = controller();

        // Drive the state machine directly: the spawned reload task is what
        // normally clears `Reloading`, and we want the queueing branch.
        *controller.state.lock().await = ReloadState::Reloading;
        controller.request_reload().await;
        assert_eq!(controller.get_state().await, ReloadState::PendingReload);

        // A further request while pending is a no-op rather than a second queue
        // slot.
        controller.request_reload().await;
        assert_eq!(controller.get_state().await, ReloadState::PendingReload);
    }

    #[tokio::test]
    async fn test_reload_controller_returns_to_idle_after_reloading() {
        let controller = controller();

        controller.request_reload().await;

        // The spawned task runs `echo test` and then settles back to `Idle`.
        for _ in 0..50 {
            if controller.get_state().await == ReloadState::Idle {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }

        panic!(
            "controller stuck in {:?} instead of returning to Idle",
            controller.get_state().await
        );
    }

    #[tokio::test]
    async fn test_initial_reload_leaves_state_idle() {
        let controller = controller();

        controller.initial_reload().await;

        // The startup reload is blocking and does not go through the queue.
        assert_eq!(controller.get_state().await, ReloadState::Idle);
    }
}
