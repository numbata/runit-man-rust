use actix_web::{web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_web::dev::ServiceRequest;
use config::AppConfig;
use env_logger::{Builder, Target};
use clap::Parser;

mod services;
mod manage_service;
mod service_logs;
mod service_info;
mod config;
mod installer;

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
    // Set up logging to STDOUT
    Builder::new()
        .target(Target::Stdout) // Direct output to stdout
        .filter_level(args.log_level.parse().expect("Invalid log level"))
        .init();

    HttpServer::new(move || {
        let auth = HttpAuthentication::basic(basic_auth_validator);

        App::new()
            .app_data(web::Data::new(config.clone()))
            .wrap(auth) // Always wrap, validator handles bypass if no credentials
            .route("/", web::get().to(|| async { "Hello, runit-man!" }))
            .route("/services", web::get().to(services::list_services))
            .route("/services/{name}", web::get().to(manage_service::get_service_info))
            .route("/services/{name}/{action}", web::post().to(manage_service::manage_service))
            .route("/services/{name}/log", web::get().to(service_logs::service_logs))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
