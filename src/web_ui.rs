use actix_web::{web, HttpResponse, Responder};
use tera::{Context, Tera};

use crate::services;
use crate::service_info::ServiceInfo;
use crate::config::AppConfig;

/// Renders the service management page
pub async fn render_service_list(config: web::Data<AppConfig>, tera: web::Data<Tera>) -> impl Responder {
    let services = services::fetch_service_list(&config.services_dir);

    let mut context = Context::new();
    context.insert("services", &services);

    match tera.render("web/index.html", &context) {
        Ok(rendered) => HttpResponse::Ok()
            .content_type("text/html")
            .body(rendered),
        Err(_err) => {
            HttpResponse::InternalServerError()
                .body("Internal Server Error")
        }
    }
}

pub async fn render_service_log(path: web::Path<String>, tera: web::Data<Tera>) -> impl Responder {
    let service_info = ServiceInfo::get_status(&path.into_inner()).unwrap();
    let mut context = Context::new();
    context.insert("service", &service_info);

    match tera.render("web/logs.html", &context) {
        Ok(rendered) => HttpResponse::Ok()
            .content_type("text/html")
            .body(rendered),
        Err(_err) => {
            HttpResponse::InternalServerError()
                .body("Internal Server Error")
        }
    }
}