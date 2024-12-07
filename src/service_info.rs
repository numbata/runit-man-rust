use std::process::Command;
use std::error::Error;
use log::{info, error, warn};

pub struct LogInfo {
    pub name: String,
    pub status: String,
    pub pid: Option<u32>,
    pub uptime: Option<u64>,
}

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

    // Return the log file path if service is running
    pub fn log_directory(&self) -> Option<String> {
        if !self.is_running() {
            return None;
        }

        // Let's check if the command under the /proc/<pid>/cmdline is the same as the service name
        let cmdline = std::fs::read_to_string(format!("/proc/{}/comm", self.pid.unwrap()));
        if cmdline.unwrap() == *"svlogd" {
            return None;
        }

        self.svlogd_log_directory()
    }

    pub fn svlogd_log_directory(&self) -> Option<String> {
        let log_file = format!("/proc/{}/cmdline", self.pid.unwrap());
        let cmdline = std::fs::read_to_string(log_file);

        match cmdline {
            Ok(cmdline) => Some(cmdline.split('\0').filter(|s| !s.is_empty()).last().unwrap().to_string()),
            Err(_) => None,
        }
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
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    info!("Service status fetched successfully: {}", output_str);

                    let name = regex::escape(name);
                    let re = regex::Regex::new(&format!(r"(?<status>[^\:]+): {}: \(pid (?<pid>\d+)\) (?<uptime>\d+)s;?(?:\ (?<log_status>[^\:]+): (?<log_name>[^:]+): \(pid (?<log_pid>\d+)\) (?<log_uptime>\d+)s)?", name)).unwrap();
                    let captures = re.captures(&output_str).unwrap();

                    let status = captures.name("status").map(|m| m.as_str().to_string());
                    let pid = captures.name("pid").map(|m| m.as_str().parse::<u32>().unwrap());
                    let uptime = captures.name("uptime").map(|m| m.as_str().parse::<u64>().unwrap());
                    let log_pid = captures.name("log_pid").map(|m| m.as_str().parse::<u32>().unwrap());
                    let log_status = captures.name("log_status").map(|m| m.as_str().to_string());
                    let log_uptime = captures.name("log_uptime").map(|m| m.as_str().parse::<u64>().unwrap());
                    let log_name = captures.name("log_name").map(|m| m.as_str().to_string());

                    let log = log_name.map(|log_name| LogInfo::new(
                        log_name,
                        log_status.unwrap(),
                        log_pid,
                        log_uptime,
                    ));

                    Ok(ServiceInfo::new(
                        name.to_string(),
                        status.unwrap(),
                        pid,
                        uptime,
                        log,
                    ))
                } else {
                    warn!("Service is not running: {}", name);
                    Ok(ServiceInfo::new(name.to_string(), "down".to_string(), None, None, None))
                }
            }
            Err(e) => {
                error!("Failed to fetch status for service {}: {}", name, e);
                Err(Box::new(e))
            }
        }
    }
}