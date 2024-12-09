use actix_web::{web, HttpResponse, Responder};
use std::io::BufReader;
use std::fs::File;
use log::info;
use serde::Deserialize;
use serde_json::json;
use rev_lines::RevLines;

use crate::service_info::ServiceInfo;

#[derive(Debug, Deserialize)]
pub struct LogQuery {
    lines: Option<usize>,
}

fn read_last_lines(file_path: &str, num_lines: usize) -> std::io::Result<String> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let rev_lines = RevLines::new(reader);

    // rev_lines yields lines from the end. Take the desired number of lines.
    let mut collected: Vec<String> = rev_lines.take(num_lines)
        .filter_map(Result::ok)
        .collect();
    collected.reverse();

    Ok(collected.join("\n"))
}

pub async fn service_logs(path: web::Path<String>, query: web::Query<LogQuery>) -> impl Responder {
    let service_name = path.into_inner();
    let service_info = ServiceInfo::get_status(&service_name).unwrap();

    if service_info.log.is_none() {
        return HttpResponse::NotFound().body(format!("Can't find logs for service {}. Probably log service is not running", service_name));
    }

    let log_path = service_info.log.unwrap().log_directory().unwrap();
    let current_log_path = format!("{}/current", log_path);

    let lines = query.lines.unwrap_or(50);

    info!("Preparing for reading last {} lines from log file {}", lines, current_log_path);
    match read_last_lines(&current_log_path, lines) {
        Ok(logs) => HttpResponse::Ok().json(json!({ "logs": logs })),
        Err(_) => HttpResponse::NotFound().json(json!({ "error": format!("Could not read logs for service {}", service_name) })),
    }
}