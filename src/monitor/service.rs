use crate::config::{helper::read, schema::CommandType};
use std::collections::HashSet;
use std::error::Error;
use std::process::Command;
use std::path::Path;

pub struct ServiceTracker {
    pub service_pids: HashSet<u32>,
    pub service_commands: Vec<String>,
    pub service_bin_paths: Vec<String>,
}

impl ServiceTracker {
    pub fn new() -> Self {
        ServiceTracker {
            service_pids: HashSet::new(),
            service_commands: Vec::new(),
            service_bin_paths: Vec::new(),
        }
    }

    /// Load service commands and binaries from rustywatch.yaml file
    pub fn load_services(&mut self, config_path: &str) -> Result<(), Box<dyn Error>> {
        if !Path::new(config_path).exists() {
            return Ok(());
        }

        let config = read(config_path.to_string())?;
        
        // Clear any existing commands and paths
        self.service_commands.clear();
        self.service_bin_paths.clear();
        
        for workspace in config.workspaces {
            // Store command parts
            match &workspace.cmd {
                CommandType::Single(cmd) => {
                    // Store the exact command
                    self.service_commands.push(cmd.clone());
                    
                    // Split by semicolons to get individual commands
                    for part in cmd.split(';') {
                        let trimmed = part.trim();
                        if !trimmed.is_empty() {
                            self.service_commands.push(trimmed.to_string());
                        }
                    }
                },
                CommandType::Multiple(cmds) => {
                    // Store each command exactly as defined
                    for cmd in cmds {
                        self.service_commands.push(cmd.clone());
                        
                        // Split by semicolons to get individual commands
                        for part in cmd.split(';') {
                            let trimmed = part.trim();
                            if !trimmed.is_empty() && trimmed != cmd.trim() {
                                self.service_commands.push(trimmed.to_string());
                            }
                        }
                    }
                }
            };
            
            // If binary path is specified, add it
            if let Some(bin_path) = &workspace.bin_path {
                self.service_bin_paths.push(bin_path.clone());
                
                // Also add the binary name by itself
                if let Some(file_name) = Path::new(bin_path).file_name() {
                    if let Some(name) = file_name.to_str() {
                        self.service_bin_paths.push(name.to_string());
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Find processes related to our services based on exact commands from config
    pub fn find_service_processes(&mut self) {
        // Clear current PIDs
        self.service_pids.clear();
        
        // Use 'ps' command to find processes
        let output = Command::new("ps")
            .args(&["aux"])
            .output()
            .expect("Failed to execute ps command");
            
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for line in output_str.lines().skip(1) { // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 11 {
                continue;
            }
            
            let pid_str = parts[1];
            let cmd = parts[10..].join(" ");
            
            // Check if this process matches any of our exact commands or binary paths
            let is_command_match = self.service_commands.iter().any(|service_cmd| {
                cmd.contains(service_cmd)
            });
            
            let is_bin_match = self.service_bin_paths.iter().any(|bin_path| {
                cmd.contains(bin_path)
            });
            
            if is_command_match || is_bin_match {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    self.service_pids.insert(pid);
                }
            }
        }
    }
    
    /// Check if a process belongs to one of our services
    pub fn is_service_process(&self, pid: &u32) -> bool {
        self.service_pids.contains(pid)
    }
}
