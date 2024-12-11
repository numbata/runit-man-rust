use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub services_dir: String,
    pub username: Option<String>,
    pub password: Option<String>,
}