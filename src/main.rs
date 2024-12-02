use actix_web::{web, App, HttpServer, HttpResponse, Responder};

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello, runit-man!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index)) // Adds a route for the root URL
    })
    .bind("127.0.0.1:8080")? // Binds to localhost on port 8080
    .run()
    .await
}
