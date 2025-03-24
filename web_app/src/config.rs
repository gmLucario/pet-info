//! Abstraction of all the values needed to setup the app

use envconfig::Envconfig;
use lazy_static::lazy_static;

/// Enviroment variables used to set specific instances in
/// the application
#[derive(Envconfig, Clone)]
pub struct AppConfig {
    pub env: String,

    /// Database host value
    pub db_host: String,

    /// Database password to encrypt db data
    pub db_pass_encrypt: String,

    /// Host, which web app will be binded
    pub wep_server_host: String,

    /// Port Host, which web app will be binded
    pub wep_server_port: u64,

    pub mercado_pago_public_key: String,

    pub mercado_token: String,

    pub whatsapp_business_phone_number_id: u64,
    pub whatsapp_business_auth: String,

    pub aws_sfn_arn_wb_notifications: String,

    /// Google oauth envs
    pub google_oauth_client_id: String,

    /// Google oauth envs
    pub google_oauth_project_id: String,

    /// Google oauth envs
    pub google_oauth_auth_uri: String,

    /// Google oauth envs
    pub google_oauth_token_uri: String,

    /// Google oauth envs
    pub google_oauth_auth_provider_x509_cert_url: String,

    /// Google oauth envs
    pub google_oauth_client_secret: String,
}

impl AppConfig {
    pub fn is_prod(&self) -> bool {
        self.env.to_lowercase() == "prod"
    }

    pub fn url_host(&self) -> String {
        if self.is_prod() {
            return self.wep_server_host.to_string();
        }

        format!(
            "{host}:{port}",
            host = self.wep_server_host,
            port = self.wep_server_port
        )
    }

    pub fn wep_server_protocol(&self) -> String {
        if self.is_prod() {
            return "https".into();
        }
        return "http".into();
    }

    pub fn whatsapp_send_msg_endpoint(&self) -> String {
        format!(
            "https://graph.facebook.com/v22.0/{id}/messages",
            id = self.whatsapp_business_phone_number_id
        )
    }
}

lazy_static! {
    pub static ref APP_CONFIG: AppConfig = AppConfig::init_from_env().unwrap();
}

lazy_static! {
    pub static ref OTP_SECRET: uuid::Uuid = uuid::Uuid::new_v4();
}
