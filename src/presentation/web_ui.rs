use actix_web::{web, HttpResponse, Responder};
use tera::{Context, Tera};

use crate::application::service_info::ServiceInfo;

pub async fn render_service_list(tera: web::Data<Tera>) -> impl Responder {
    let context = Context::new();

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
