use crate::application::service_info::ServiceInfo;
use std::fs;
use log::info;

pub fn fetch_service_list(services_dir: &str) -> Vec<ServiceInfo> {
    let mut service_list = Vec::new();

    if let Ok(entries) = fs::read_dir(services_dir) {
        for entry in entries.flatten() {
            let service_name = entry.file_name().to_string_lossy().into_owned();
            info!("Service found: {}", service_name);
            if let Ok(service_info) = ServiceInfo::get_status(&service_name) {
                service_list.push(service_info);
            }
        }
    }

    service_list
}
