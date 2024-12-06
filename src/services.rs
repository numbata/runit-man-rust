use crate::service_info::ServiceInfo;
use crate::config::AppConfig;
use actix_web::{web, HttpResponse, Responder};
use std::fs;

pub async fn list_services(config: web::Data<AppConfig>) -> impl Responder {
    let services_dir = &config.services_dir;
    let mut service_list = Vec::new();

    if let Ok(entries) = fs::read_dir(services_dir) {
        for entry in entries.flatten() {
            let service_name = entry.file_name().into_string().unwrap();
            if let Ok(service_info) = ServiceInfo::get_status(&service_name) {
                service_list.push(service_info);
            }
        }
    }

    let json_response = serde_json::json!(service_list.iter().map(|s| s.as_json()).collect::<Vec<_>>());
    HttpResponse::Ok().json(json_response)
}