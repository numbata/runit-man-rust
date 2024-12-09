use crate::service_info::ServiceInfo;

use std::os::unix::fs::symlink;
use std::fs::remove_file;
use actix_web::{web, HttpResponse, Responder};
use std::process::{Command, Output};
use std::io;


#[derive(Debug)]
enum ServiceAction {
    Start,
    Stop,
    Restart,
    Enable,
    Disable,
}

impl ServiceAction {
    fn from_str(action: &str) -> Option<Self> {
        match action {
            "start" => Some(ServiceAction::Start),
            "stop" => Some(ServiceAction::Stop),
            "restart" => Some(ServiceAction::Restart),
            "enable" => Some(ServiceAction::Enable),
            "disable" => Some(ServiceAction::Disable),
            _ => None,
        }
    }
}

fn execute_command(command: &mut Command) -> Result<Output, io::Error> {
    let output = command.output()?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Command execution failed: {:?} with status: {}",
                command,
                output.status
            ),
        ))
    }
}

pub async fn get_service_info(path: web::Path<String>) -> impl Responder {
    let service_name = path.into_inner();
    match ServiceInfo::get_status(&service_name) {
        Ok(service_info) => HttpResponse::Ok().json(service_info.as_json()),
        Err(_) => HttpResponse::NotFound().body(format!("Service {} not found", service_name)),
    }
}

pub async fn manage_service(path: web::Path<(String, String)>) -> impl Responder {
    let (service_name, action) = path.into_inner();
    let action = ServiceAction::from_str(&action);

    match action {
        Some(ServiceAction::Start) => {
            match execute_command(Command::new("sv").arg("up").arg(&service_name)) {
                Ok(_) => HttpResponse::Ok().body(format!("Service {} started.", service_name)),
                Err(e) => HttpResponse::InternalServerError().body(format!("Failed to start service {}: {}", service_name, e)),
            }
        },
        Some(ServiceAction::Stop) => {
            match execute_command(Command::new("sv").arg("down").arg(&service_name)) {
                Ok(_) => HttpResponse::Ok().body(format!("Service {} stopped.", service_name)),
                Err(e) => HttpResponse::InternalServerError().body(format!("Failed to stop service {}: {}", service_name, e)),
            }
        },
        Some(ServiceAction::Restart) => {
            match execute_command(Command::new("sv").arg("restart").arg(&service_name)) {
                Ok(_) => HttpResponse::Ok().body(format!("Service {} restarted.", service_name)),
                Err(e) => HttpResponse::InternalServerError().body(format!("Failed to restart service {}: {}", service_name, e)),
            }
        },
        Some(ServiceAction::Enable) => {
            let source = format!("/etc/sv/{}", service_name);
            let target = format!("/etc/service/{}", service_name);

            match symlink(source, target) {
                Ok(_) => HttpResponse::Ok().body(format!("Service {} enabled.", service_name)),
                Err(e) => HttpResponse::InternalServerError().body(format!("Failed to enable service {}: {}", service_name, e)),
            }
        },
        Some(ServiceAction::Disable) => {
            let target = format!("/etc/service/{}", service_name);

            match remove_file(target) {
                Ok(_) => HttpResponse::Ok().body(format!("Service {} disabled.", service_name)),
                Err(e) => HttpResponse::InternalServerError().body(format!("Failed to disable service {}: {}", service_name, e)),
            }
        },
        None => HttpResponse::BadRequest().body("Invalid action"),
    }
}