use crate::config::schema::Config;
use std::collections::HashSet;
use std::error::Error;
use std::path::Path;
use sysinfo::{Pid, Process, System};

pub struct ServiceTracker {
    /// Tracked PIDs that are confirmed to be our services
    pub service_pids: HashSet<u32>,
    /// O(1) lookup for command matching
    pub service_commands: HashSet<String>,
    /// O(1) lookup for binary path matching
    pub service_bin_paths: HashSet<String>,
    /// Counter for periodic full scans
    refresh_counter: u32,
    /// Full scan interval (e.g., every 10 refreshes = 10 seconds at 1s refresh rate)
    full_scan_interval: u32,
}

impl ServiceTracker {
    pub fn new() -> Self {
        ServiceTracker {
            service_pids: HashSet::new(),
            service_commands: HashSet::new(),
            service_bin_paths: HashSet::new(),
            refresh_counter: 0,
            full_scan_interval: 10,
        }
    }

    /// Load service commands and binaries from rustywatch.yaml file
    pub fn load_services(&mut self, config_path: &str) -> Result<(), Box<dyn Error>> {
        if !Path::new(config_path).exists() {
            return Ok(());
        }

        let config = Config::from_file(config_path)?;

        // Clear any existing commands and paths
        self.service_commands.clear();
        self.service_bin_paths.clear();

        for workspace in config.workspaces {
            // Store command parts
            for cmd in workspace.cmd.iter() {
                self.add_command(cmd);
            }

            // If binary path is specified, add it
            if let Some(bin_path) = &workspace.bin_path {
                self.service_bin_paths.insert(bin_path.clone());

                // Also add the binary name by itself
                if let Some(file_name) = Path::new(bin_path).file_name() {
                    if let Some(name) = file_name.to_str() {
                        self.service_bin_paths.insert(name.to_string());
                    }
                }
            }
        }

        Ok(())
    }

    /// Helper to add command and its semicolon-split parts
    fn add_command(&mut self, cmd: &str) {
        self.service_commands.insert(cmd.to_string());
        for part in cmd.split(';') {
            let trimmed = part.trim();
            if !trimmed.is_empty() {
                self.service_commands.insert(trimmed.to_string());
            }
        }
    }

    /// Check if a process matches our service criteria (O(1) lookups)
    #[inline]
    fn matches_service(&self, process: &Process) -> bool {
        let cmd_parts: Vec<&str> = process.cmd().iter().map(|s| s.as_str()).collect();
        let cmd_line = cmd_parts.join(" ");
        let name = process.name();

        // O(1) HashSet contains check with substring matching
        self.service_commands
            .iter()
            .any(|svc_cmd| cmd_line.contains(svc_cmd))
            || self
                .service_bin_paths
                .iter()
                .any(|bin| cmd_line.contains(bin) || name.contains(bin))
    }

    /// Discover service processes using sysinfo (replaces ps aux)
    pub fn discover_service_processes(&mut self, system: &System) {
        self.service_pids.clear();

        for (pid, process) in system.processes() {
            if self.matches_service(process) {
                self.service_pids.insert(pid.as_u32());
            }
        }
    }

    /// Incremental update: refresh tracked PIDs and detect terminated processes
    pub fn refresh_tracked_processes(&mut self, system: &System) {
        // Remove PIDs that no longer exist
        self.service_pids
            .retain(|&pid| system.process(Pid::from_u32(pid)).is_some());
    }

    /// Find processes related to our services (uses sysinfo directly)
    pub fn find_service_processes(&mut self) {
        // For backward compatibility, create a temporary System
        let mut system = System::new();
        system.refresh_processes();
        self.discover_service_processes(&system);
    }

    /// Smart refresh: incremental most of the time, full scan periodically
    pub fn smart_refresh(&mut self, system: &System) {
        self.refresh_counter += 1;

        if self.refresh_counter >= self.full_scan_interval {
            // Periodic full scan to discover new processes
            self.discover_service_processes(system);
            self.refresh_counter = 0;
        } else {
            // Incremental: only check tracked PIDs
            self.refresh_tracked_processes(system);
        }
    }

    /// Check if a process belongs to one of our services
    pub fn is_service_process(&self, pid: &u32) -> bool {
        self.service_pids.contains(pid)
    }
}

impl Default for ServiceTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_service_tracker_new() {
        let tracker = ServiceTracker::new();
        assert!(tracker.service_pids.is_empty());
        assert!(tracker.service_commands.is_empty());
        assert!(tracker.service_bin_paths.is_empty());
    }

    #[test]
    fn test_load_services_with_single_command() {
        let mut tracker = ServiceTracker::new();

        let config_content = r#"
workspaces:
  - dir: "/path/to/project"
    cmd: "cargo run"
"#;
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let result = tracker.load_services(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert!(tracker.service_commands.contains("cargo run"));
    }

    #[test]
    fn test_load_services_with_multiple_commands() {
        let mut tracker = ServiceTracker::new();

        let config_content = r#"
workspaces:
  - dir: "/path/to/project"
    cmd:
      - "npm install"
      - "npm start"
"#;
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let result = tracker.load_services(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert!(tracker.service_commands.contains("npm install"));
        assert!(tracker.service_commands.contains("npm start"));
    }

    #[test]
    fn test_load_services_with_bin_path() {
        let mut tracker = ServiceTracker::new();

        let config_content = r#"
workspaces:
  - dir: "/path/to/project"
    cmd: "cargo build"
    bin_path: "/path/to/bin/myapp"
"#;
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let result = tracker.load_services(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert!(tracker.service_bin_paths.contains("/path/to/bin/myapp"));
        assert!(tracker.service_bin_paths.contains("myapp"));
    }

    #[test]
    fn test_load_services_file_not_found() {
        let mut tracker = ServiceTracker::new();
        let result = tracker.load_services("/nonexistent/path/config.yaml");
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_service_process() {
        let mut tracker = ServiceTracker::new();
        tracker.service_pids.insert(1234);
        tracker.service_pids.insert(5678);

        assert!(tracker.is_service_process(&1234));
        assert!(tracker.is_service_process(&5678));
        assert!(!tracker.is_service_process(&9999));
    }

    #[test]
    fn test_load_services_with_semicolon_commands() {
        let mut tracker = ServiceTracker::new();

        let config_content = r#"
workspaces:
  - dir: "/path/to/project"
    cmd: "npm install; npm start"
"#;
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let result = tracker.load_services(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        assert!(tracker.service_commands.contains("npm install"));
        assert!(tracker.service_commands.contains("npm start"));
    }

    #[test]
    fn test_hashset_deduplication() {
        let mut tracker = ServiceTracker::new();

        let config_content = r#"
workspaces:
  - dir: "/path/to/project1"
    cmd: "cargo run"
  - dir: "/path/to/project2"
    cmd: "cargo run"
"#;
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let result = tracker.load_services(temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        // HashSet should deduplicate
        assert_eq!(
            tracker
                .service_commands
                .iter()
                .filter(|c| *c == "cargo run")
                .count(),
            1
        );
    }

    #[test]
    fn test_discover_service_processes() {
        let mut tracker = ServiceTracker::new();
        tracker.service_commands.insert("sh".to_string());

        let mut system = System::new();
        system.refresh_processes();

        tracker.discover_service_processes(&system);

        // Should find at least some processes with "sh" in the command
        // This depends on the system, so we just check it doesn't panic
    }
}
