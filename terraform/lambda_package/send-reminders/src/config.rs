use envconfig::Envconfig;
use std::sync::LazyLock;

#[derive(Envconfig, Clone)]
pub struct AppConfig {
    pub whatsapp_business_phone_number_id: u64,
    pub whatsapp_business_auth: String,
    pub step_function_arn: String,
    pub web_app_api_url: String,
    pub internal_api_secret: String,
}

impl AppConfig {
    pub fn whatsapp_send_msg_endpoint(&self) -> String {
        format!(
            "https://graph.facebook.com/v22.0/{id}/messages",
            id = self.whatsapp_business_phone_number_id
        )
    }
}

pub static APP_CONFIG: LazyLock<AppConfig> = LazyLock::new(|| AppConfig::init_from_env().unwrap());
