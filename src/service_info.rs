use std::process::Command;
use std::error::Error;
use serde::Serialize;
use log::{info, error, warn};

#[derive(Serialize)]
pub struct LogInfo {
    pub name: String,
    pub status: String,
    pub pid: Option<u32>,
    pub uptime: Option<u64>,
}

#[derive(Serialize)]
pub struct ServiceInfo {
    pub name: String,
    pub status: String,
    pub pid: Option<u32>,
    pub uptime: Option<u64>,
    pub log: Option<LogInfo>,
}

impl LogInfo {
    pub fn new(name: String, status: String, pid: Option<u32>, uptime: Option<u64>) -> Self {
        Self {
            name,
            status,
            pid,
            uptime,
        }
    }

    pub fn is_running(&self) -> bool {
        self.status == "run"
    }

    pub fn log_directory(&self) -> Option<String> {
        if !self.is_running() {
            return None;
        }

        let pid = self.pid?;
        let cmdline_path = format!("/proc/{}/comm", pid);

        let cmdline = std::fs::read_to_string(&cmdline_path)
            .map_err(|e| {
                log::warn!(
                    "Failed to read cmdline for pid {}: {}",
                    pid,
                    e
                );
                e
            })
            .ok()?;

        if cmdline.trim() == "svlogd" {
            self.svlogd_log_directory()
        } else {
            None
        }
    }

    pub fn svlogd_log_directory(&self) -> Option<String> {
        let log_file_path = format!("/proc/{}/cmdline", self.pid?);
        let cmdline = std::fs::read_to_string(&log_file_path).map_err(|e| {
            log::warn!(
                "Failed to read svlogd log directory for pid {}: {}",
                self.pid.unwrap(),
                e
            );
            e
        }).ok()?;

        Some(
            cmdline
                .split('\0')
                .filter(|s| !s.is_empty())
                .last()?
                .to_string(),
        )
    }

    pub fn as_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "status": self.status,
            "pid": self.pid,
            "uptime": self.uptime,
            "directory": self.log_directory(),
        })
    }
}

impl ServiceInfo {
    // Constructor for creating a new ServiceInfo
    pub fn new(name: String, status: String, pid: Option<u32>, uptime: Option<u64>, log: Option<LogInfo>) -> Self {
        Self {
            name,
            status,
            pid,
            uptime,
            log,
        }
    }

    pub fn is_running(&self) -> bool {
        self.status == "run"
    }

    // Method to serialize into JSON format
    pub fn as_json(&self) -> serde_json::Value {
        info!("Exposing service info: {}", self.name);
        serde_json::json!({
            "name": self.name,
            "is_running": self.is_running(),
            "status": self.status,
            "pid": self.pid,
            "uptime": self.uptime,
            "log": self.log.as_ref().map(|log| log.as_json()),
        })
    }

    pub fn get_status(name: &str) -> Result<Self, Box<dyn Error>> {
        info!("Fetching status for service: {}", name);

        let output = Command::new("sv")
            .arg("status")
            .arg(name)
            .output()
            .map_err(|e| {
                error!("Failed to execute sv command for {}: {}", name, e);
                e
            })?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            info!("Service status fetched successfully: {}", output_str);

            let re = regex::Regex::new(
                &format!(
                    r"(?<status>[^:]+): {}:(?: \(pid (?<pid>\d+)\))? (?<uptime>\d+)s(?:; (?<log_status>[^:]+): (?<log_name>[^:]+):(?: \(pid (?<log_pid>\d+)\))? (?<log_uptime>\d+)s)?",
                    regex::escape(name)
                ),
            )?;
            info!("Regex: {}", re);

            let captures = re.captures(&output_str).ok_or_else(|| {
                log::warn!("Regex did not match output: {}", output_str);
                "Regex capture failed"
            })?;

            let status = captures.name("status").map(|m| m.as_str().to_string());
            let pid = captures.name("pid").map(|m| m.as_str().parse::<u32>().unwrap());
            let uptime = captures.name("uptime").map(|m| m.as_str().parse::<u64>().unwrap());
            let log = captures.name("log_name").map(|log_name| {
                LogInfo::new(
                    log_name.as_str().to_string(),
                    captures["log_status"].to_string(),
                    captures.name("log_pid").and_then(|m| m.as_str().parse::<u32>().ok()),
                    captures.name("log_uptime").and_then(|m| m.as_str().parse::<u64>().ok()),
                )
            });

            Ok(ServiceInfo::new(
                name.to_string(),
                status.unwrap_or_else(|| "unknown".to_string()),
                pid,
                uptime,
                log,
            ))
        } else {
            warn!("Service is not running: {}", name);
            Ok(ServiceInfo::new(name.to_string(), "down".to_string(), None, None, None))
        }
    }
}