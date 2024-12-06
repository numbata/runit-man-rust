use std::process::Command;
use std::error::Error;
use log::{info, error, warn};

pub struct ServiceInfo {
    pub name: String,
    pub is_running: bool,
    pub pid: Option<u32>,
    pub uptime: Option<u64>,
}

impl ServiceInfo {
    // Constructor for creating a new ServiceInfo
    pub fn new(name: String, is_running: bool, pid: Option<u32>, uptime: Option<u64>) -> Self {
        Self {
            name,
            is_running,
            pid,
            uptime,
        }
    }

    // Method to serialize into JSON format
    pub fn as_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "is_running": self.is_running,
            "pid": self.pid,
            "uptime": self.uptime,
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

                    let pid = output_str.split_whitespace().find(|part| part.starts_with("(pid"))
                        .and_then(|pid_str| pid_str.trim_matches(&['(', ')', 'p', 'i', 'd'][..]).parse::<u32>().ok());

                    let uptime = output_str.split_whitespace().find(|part| part.ends_with("s"))
                        .and_then(|uptime_str| uptime_str.trim_end_matches("s").parse::<u64>().ok());

                    Ok(ServiceInfo::new(name.to_string(), true, pid, uptime))
                } else {
                    warn!("Service is not running: {}", name);
                    Ok(ServiceInfo::new(name.to_string(), false, None, None))
                }
            }
            Err(e) => {
                error!("Failed to fetch status for service {}: {}", name, e);
                Err(Box::new(e))
            }
        }
    }
}