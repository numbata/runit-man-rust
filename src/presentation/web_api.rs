use actix_web::{web, HttpResponse};
use actix_web::Responder;
use log::info;
use serde::Deserialize;
use serde_json::json;

use crate::config::app_config::AppConfig;
use crate::domain::service;
use crate::application::manage_service;
use crate::domain::service_logs;
use crate::application::service_info::ServiceInfo;

#[derive(Debug, Deserialize)]
pub struct LogQuery {
    lines: Option<usize>,
}

pub async fn render_service_info(path: web::Path<String>) -> impl Responder {
    let service_name = path.into_inner();
    match ServiceInfo::get_status(&service_name) {
        Ok(service_info) => HttpResponse::Ok().json(service_info.as_json()),
        Err(_) => HttpResponse::NotFound().body(format!("Service {} not found", service_name)),
    }
}

pub async fn render_service_list(config: web::Data<AppConfig>) -> impl Responder {
    let services_dir = &config.services_dir;
    let service_list = service::fetch_service_list(services_dir);
    let json_response = json!(service_list.iter().map(|s| s.as_json()).collect::<Vec<_>>());
    info!("JSON builded: {}", json_response);
    HttpResponse::Ok().json(json_response)
}

pub async fn render_service_log(path: web::Path<String>, query: web::Query<LogQuery>) -> impl Responder {
    let service_name = path.into_inner();
    let service_info = ServiceInfo::get_status(&service_name).unwrap();
    let lines = query.lines.unwrap_or(50);

    match service_logs::service_logs(service_info, lines) {
        Ok(logs) => HttpResponse::Ok().json(json!({ "logs": logs })),
        Err(_) => HttpResponse::NotFound().json(json!({ "error": format!("Could not read logs for service {}", service_name) })),
    }
}

pub async fn manage_service(path: web::Path<(String, String)>) -> impl Responder {
    let (service_name, action) = path.into_inner();
    match manage_service::perform_service_action(&service_name, &action) {
        Ok(message) => HttpResponse::Ok().body(message),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}