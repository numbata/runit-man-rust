use crate::service_info::ServiceInfo;
use crate::config::AppConfig;
use actix_web::{web, HttpResponse, Responder};
use std::fs;
use log::info;

pub async fn list_services(config: web::Data<AppConfig>) -> impl Responder {
    let services_dir = &config.services_dir;
    let service_list = fetch_service_list(services_dir);
    let json_response = serde_json::json!(service_list.iter().map(|s| s.as_json()).collect::<Vec<_>>());
    info!("JSON builded: {}", json_response);
    HttpResponse::Ok().json(json_response)
}

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