use actix_web::{web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_web::dev::ServiceRequest;
use actix_web::HttpResponse;
use anyhow::{Context, Result};
use tera::Tera;
use include_dir::{include_dir, Dir, DirEntry};
use config::AppConfig;
use env_logger::{Builder, Target};
use log::info;
use clap::Parser;

mod services;
mod manage_service;
mod service_logs;
mod service_info;
mod config;
mod installer;
mod web_ui;

static TEMPLATES_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/templates");

async fn favicon() -> HttpResponse {
    let favicon_bytes = include_bytes!("../static/favicon.ico");
    HttpResponse::Ok()
        .content_type("image/x-icon")
        .body(&favicon_bytes[..])
}


/// Program to install a service with runit
#[derive(Parser, Debug)]
#[command(version = "1.0", about = "Installs a service with runit")]
struct Args {
    /// Install the service
    /// This flag will install the service with runit
    #[arg(long, default_value = "false")]
    install: bool,

    /// The name of the service to install
    #[arg(long, default_value = "runit-man-rust")]
    service_name: String,

    /// The directory for log files
    #[arg(long, default_value = "/var/log/runit-man")]
    log_directory: String,

    /// Set logging level
    /// This flag will set the logging level for the application
    #[arg(long, default_value = "info")]
    log_level: String,

    /// The directory for service files
    #[arg(long, default_value = "/etc/sv")]
    services_dir: String,

    /// The username for basic authentication
    #[arg(long)]
    username: Option<String>,

    /// The password for basic authentication
    #[arg(long)]
    password: Option<String>,

    // The application bind address
    #[arg(long, default_value = "0.0.0.0:8080")]
    host: String,
}

async fn basic_auth_validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    let username = credentials.user_id();
    let password = credentials.password().map_or("", |v| v);

    // Borrow the app data instead of moving the request
    if let Some(app_data) = req.app_data::<web::Data<AppConfig>>() {
        // Check if username and password are configured
        if let (Some(expected_username), Some(expected_password)) = (&app_data.username, &app_data.password) {
            if username == expected_username && password == expected_password {
                return Ok(req);
            }
        } else {
            // If credentials are not configured, bypass authentication
            return Ok(req);
        }
    }

    Err((actix_web::error::ErrorUnauthorized("Invalid username or password"), req))
}

fn load_embedded_templates() -> Result<Tera> {
    let mut tera = Tera::default();

    info!("Finding templates");
    for entry in TEMPLATES_DIR.find("**/*.html")? {
        match entry {
            DirEntry::File(file) => {
                let name = file.path().to_str()
                    .context("Invalid template path encoding")?;

                println!("Loading template: {}", name);

                if let Some(content) = file.contents_utf8() {
                    tera.add_raw_template(name, content)
                        .context(format!("Failed to add template: {}", name))?;
                } else {
                    eprintln!("Skipping non-UTF8 template: {}", file.path().display());
                }
            },
            DirEntry::Dir(_) => continue,
        }
    }

    Ok(tera)
}

fn handle_installation(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    installer::install_service(args)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    if args.install {
        if let Err(e) = handle_installation(&args) {
            eprintln!("Failed to install service: {}", e);
            std::process::exit(1);
        }
        println!("Service installed successfully.");
        return Ok(());
    }

    // Initialize configuration
    let config = AppConfig {
        username: args.username,
        password: args.password,
        services_dir: args.services_dir,
    };

    Builder::new()
        .target(Target::Stdout) // Direct output to stdout
        .filter_level(args.log_level.parse().expect("Invalid log level"))
        .init();
    info!("Finding templates");
    for entry in TEMPLATES_DIR.files() {
        info!("Found file: {}", entry.path().display());
    }
    info!("Finding templates");


    info!("CARGO_MANIFEST_DIR: {}", env!("CARGO_MANIFEST_DIR"));

    HttpServer::new(move || {
        let auth = HttpAuthentication::basic(basic_auth_validator);
        info!("Load templates");
        let tera = load_embedded_templates();

        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(tera.unwrap().clone()))
            .wrap(auth) // Always wrap, validator handles bypass if no credentials
            .route("/", web::get().to(web_ui::render_service_list))
            .route("/favicon.ico", web::get().to(favicon))
            .route("/services/{name}/log", web::get().to(web_ui::render_service_log))
            .route("/api/services", web::get().to(services::list_services))
            .route("/api/services/{name}", web::get().to(manage_service::get_service_info))
            .route("/api/services/{name}/{action}", web::post().to(manage_service::manage_service))
            .route("/api/services/{name}/log", web::get().to(service_logs::service_logs))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
