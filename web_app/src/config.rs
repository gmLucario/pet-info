//! Application configuration management with security considerations.
//!
//! This module handles all configuration values required for the application.
//! It includes secure storage indicators for sensitive configuration fields
//! and validation mechanisms to ensure proper security practices.
//!
//! # Security Notes
//! - Sensitive fields are clearly marked and should never be logged
//! - Production environments should use secure secret management systems
//! - All sensitive data should be stored using encryption at rest

use envconfig::Envconfig;
use std::sync::LazyLock;

/// Application configuration with security-aware field management.
///
/// This struct contains all environment variables used to configure the application.
/// Sensitive fields are clearly marked and include security guidance.
///
/// # Security Requirements
/// - All `SENSITIVE` fields must be stored securely (encrypted at rest)
/// - Use secret management systems in production (AWS Secrets Manager, HashiCorp Vault, etc.)
/// - Never log or expose sensitive values
/// - Rotate sensitive credentials regularly
#[derive(Envconfig, Clone)]
pub struct AppConfig {
    /// Environment name to deploy the app (NON-SENSITIVE)
    /// Values: "local", "dev", "staging", "prod"
    #[envconfig(default = "local")]
    pub env: String,

    /// Database host value (NON-SENSITIVE)
    /// Example: "sqlite:data/app.db"
    pub db_host: String,

    /// 🔒 SENSITIVE: Database password to encrypt SQLite data
    pub db_pass_encrypt: String,

    /// Host address for web server binding (NON-SENSITIVE)
    /// Example: "0.0.0.0", "localhost", "pet-info.link"
    pub wep_server_host: String,

    /// Port for web server binding (NON-SENSITIVE)
    /// Common values: 80 (HTTP), 443 (HTTPS), 8080 (dev)
    pub wep_server_port: u64,

    /// Path to SSL private key file (SENSITIVE PATH)
    /// Security: File should have 600 permissions, store path securely
    /// Example: "/etc/ssl/private/server.key"
    #[envconfig(default = "server.key")]
    pub private_key_path: String,

    /// Path to SSL certificate file (NON-SENSITIVE)
    /// Example: "/etc/ssl/certs/server.crt"
    #[envconfig(default = "server.crt")]
    pub certificate_path: String,

    /// 🔒 SENSITIVE: CSRF protection password (UUID format)
    /// Security: Generate using cryptographically secure random generator
    /// Rotation: Change on security incidents or every 6 months
    pub csrf_pass: String,

    /// 🔒 SENSITIVE: CSRF protection salt (UUID format)
    /// Security: Generate using cryptographically secure random generator  
    /// Rotation: Change with csrf_pass
    /// Access: CSRF token generation only
    pub csrf_salt: String,

    /// MercadoPago public key (SEMI-SENSITIVE)
    /// Security: Can be exposed to frontend but should be environment-specific
    /// Example: "APP_USR-12345678-090123-abcdef123456789-987654321"
    pub mercado_pago_public_key: String,

    /// 🔒 SENSITIVE: MercadoPago access token
    /// Security: Store in secure secret management system
    /// Scope: Limited to required payment operations
    pub mercado_token: String,

    /// WhatsApp Business phone number ID (SEMI-SENSITIVE)
    /// Security: Restrict access, don't log in production
    pub whatsapp_business_phone_number_id: u64,

    /// 🔒 SENSITIVE: WhatsApp Business authentication token
    /// Security: Store in secure secret management system
    pub whatsapp_business_auth: String,

    /// AWS Step Functions ARN for notifications (SEMI-SENSITIVE)
    /// Security: Contains account information, restrict access
    /// Example: "arn:aws:states:us-east-1:123456789012:stateMachine:notifications"
    pub aws_sfn_arn_wb_notifications: String,

    /// Google OAuth client ID (SEMI-SENSITIVE)
    /// Security: Can be exposed to frontend but should be environment-specific
    pub google_oauth_client_id: String,

    /// Google OAuth project ID (NON-SENSITIVE)
    /// Example: "pet-info-app-prod"
    pub google_oauth_project_id: String,

    /// Google OAuth authorization URI (NON-SENSITIVE)
    /// Standard value: "https://accounts.google.com/o/oauth2/auth"
    pub google_oauth_auth_uri: String,

    /// Google OAuth token URI (NON-SENSITIVE)
    /// Standard value: "https://oauth2.googleapis.com/token"
    pub google_oauth_token_uri: String,

    /// Google OAuth certificate URL (NON-SENSITIVE)
    /// Standard value: "https://www.googleapis.com/oauth2/v1/certs"
    pub google_oauth_auth_provider_x509_cert_url: String,

    /// 🔒 SENSITIVE: Google OAuth client secret
    /// Security: Store in secure secret management system
    pub google_oauth_client_secret: String,
}

impl AppConfig {
    /// Checks if running in production environment
    pub fn is_prod(&self) -> bool {
        self.env.to_lowercase() == "prod"
    }

    /// Gets the server URL host with port for non-production environments
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

    /// Gets the appropriate protocol (HTTP/HTTPS) based on environment
    pub fn wep_server_protocol(&self) -> String {
        if self.is_prod() {
            return "https".into();
        }
        "http".into()
    }

    /// Constructs the complete base URL for the application
    pub fn base_url(&self) -> String {
        format!("{}://{}", self.wep_server_protocol(), self.url_host())
    }

    /// Constructs the WhatsApp Business API endpoint for sending messages
    pub fn whatsapp_send_msg_endpoint(&self) -> String {
        format!(
            "https://graph.facebook.com/v22.0/{id}/messages",
            id = self.whatsapp_business_phone_number_id
        )
    }
}

/// Global application configuration instance with validation
///
/// This configuration is validated on first access to ensure security requirements.
/// If validation fails, the application will panic with a descriptive error message.
pub static APP_CONFIG: LazyLock<AppConfig> = LazyLock::new(|| {
    AppConfig::init_from_env()
        .expect("Failed to load and validate application configuration. Check environment variables and security requirements.")
});

/// 🔒 SENSITIVE: One-Time Password secret (regenerated on each application start)
///
/// Security: This UUID is regenerated on every application restart
pub static OTP_SECRET: LazyLock<uuid::Uuid> = LazyLock::new(uuid::Uuid::new_v4);
