use envconfig::Envconfig;
use std::sync::LazyLock;

#[derive(Envconfig, Clone)]
pub struct AppConfig {
    pub env: String,
    pub db_host: String,
    pub db_pass_encrypt: String,
}

impl AppConfig {
    pub fn is_prod(&self) -> bool {
        self.env.to_lowercase() == "prod"
    }
}

pub static APP_CONFIG: LazyLock<AppConfig> = LazyLock::new(|| AppConfig::init_from_env().unwrap());
