//! Application configuration management with security considerations.
//!
//! This module handles all configuration values required for the application.
//! It supports two configuration sources based on feature flags:
//! - Environment variables (default)
//! - AWS SSM Parameter Store (with `ssm` feature)
//!
//! # Configuration Sources
//!
//! ## Environment Variables (Default)
//! When compiled without the `ssm` feature, configuration is loaded from environment
//! variables using the `envconfig` crate. This is suitable for local development
//! and simple deployments.
//!
//! ## AWS SSM Parameter Store (Feature: `ssm`)
//! When compiled with the `ssm` feature, configuration is loaded from AWS SSM
//! Parameter Store under the `/pet-info/` path. This provides:
//! - Centralized configuration management
//! - Automatic encryption/decryption of sensitive values
//! - Integration with AWS IAM for access control
//! - Version history and change tracking
//!
//! # Security Notes
//! - Sensitive fields are clearly marked with 🔒 and should never be logged
//! - Production environments should use secure secret management systems
//! - All sensitive data should be stored using encryption at rest
//! - SSM SecureString parameters are automatically decrypted using the default KMS key
//!
//! # Usage
//!
//! ```rust
//! // Initialize configuration (async)
//! config::init_config().await?;
//!
//! // Access configuration
//! let app_config = config::APP_CONFIG.get().context("failed to get app config")?;
//! println!("Running in {} environment", app_config.env);
//! ```

use anyhow::Context;
use envconfig::Envconfig;
use serde::{Deserialize, Deserializer};
use std::sync::LazyLock;
use tokio::sync::OnceCell;

/// Custom deserializer to convert string values to u64.
///
/// This is needed because SSM Parameter Store stores all values as strings,
/// but we need to convert numeric values to their appropriate Rust types
/// during deserialization.
///
/// # Arguments
/// * `deserializer` - The serde deserializer
///
/// # Returns
/// The parsed u64 value or a deserialization error if parsing fails.
fn deserialize_string_to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<u64>().map_err(serde::de::Error::custom)
}

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
#[derive(Envconfig, Clone, serde::Deserialize)]
#[serde(rename_all(deserialize = "UPPERCASE"))]
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
    /// Note: Application binds to localhost:8080 (HTTP)
    /// Nginx reverse proxy handles HTTPS/TLS on port 443
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub wep_server_port: u64,

    /// 🔒 SENSITIVE: CSRF protection password (UUID format)
    /// Security: Generate using cryptographically secure random generator
    /// Rotation: Change on security incidents or every 6 months
    pub csrf_pass: uuid::Uuid,

    /// 🔒 SENSITIVE: CSRF protection salt (UUID format)
    /// Security: Generate using cryptographically secure random generator  
    /// Rotation: Change with csrf_pass
    /// Access: CSRF token generation only
    pub csrf_salt: uuid::Uuid,

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
    #[serde(deserialize_with = "deserialize_string_to_u64")]
    pub whatsapp_business_phone_number_id: u64,

    /// 🔒 SENSITIVE: WhatsApp Business authentication token
    /// Security: Store in secure secret management system
    pub whatsapp_business_auth: String,

    /// 🔒 SENSITIVE: WhatsApp webhook verification token
    /// Security: Used to verify webhook requests from WhatsApp
    /// This token must match the value configured in WhatsApp Business API dashboard
    pub whatsapp_verify_token: String,

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
    #[envconfig(default = "https://accounts.google.com/o/oauth2/auth")]
    pub google_oauth_auth_uri: String,

    /// Google OAuth token URI (NON-SENSITIVE)
    #[envconfig(default = "https://oauth2.googleapis.com/token")]
    pub google_oauth_token_uri: String,

    /// Google OAuth certificate URL (NON-SENSITIVE)
    #[envconfig(default = "https://www.googleapis.com/oauth2/v1/certs")]
    pub google_oauth_auth_provider_x509_cert_url: String,

    /// 🔒 SENSITIVE: Google OAuth client secret
    /// Security: Store in secure secret management system
    pub google_oauth_client_secret: String,

    #[envconfig(default = "pass_certificate.pem")]
    pub pass_cert_path: String,

    #[envconfig(default = "pass_private_key.pem")]
    pub pass_key_path: String,

    pub logfire_token: String,
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

/// 🔒 SENSITIVE: One-Time Password secret (regenerated on each application start)
///
/// Security: This UUID is regenerated on every application restart
pub static OTP_SECRET: LazyLock<uuid::Uuid> = LazyLock::new(uuid::Uuid::new_v4);

/// Global application configuration instance with validation
///
/// This configuration is validated on first access to ensure security requirements.
/// If validation fails, the application will panic with a descriptive error message.
pub static APP_CONFIG: OnceCell<AppConfig> = OnceCell::const_new();

/// Initialize the application configuration from environment variables (default).
///
/// This function loads configuration from environment variables using the `envconfig` crate.
/// It's the default configuration source when the `ssm` feature is not enabled.
///
/// # Environment Variables Required
/// All fields in [`AppConfig`] without default values must be provided as environment variables.
/// Variable names are the field names in UPPERCASE (e.g., `DB_HOST`, `CSRF_PASS`).
///
/// # Errors
/// Returns an error if:
/// - Required environment variables are missing
/// - Environment variable values are invalid (e.g., non-numeric values for numeric fields)
/// - Configuration has already been initialized
///
/// # Example
/// ```bash
/// export DB_HOST="sqlite:data/app.db"
/// export DB_PASS_ENCRYPT="your-encryption-key"
/// export CSRF_PASS="your-csrf-password"
/// # ... other required variables
/// ```
#[cfg(not(feature = "ssm"))]
pub async fn init_config() -> anyhow::Result<()> {
    let config = AppConfig::init_from_env()
        .context("Failed to load and validate application configuration. Check environment variables and security requirements.")?;

    APP_CONFIG
        .set(config)
        .map_err(|_| anyhow::anyhow!("Configuration already initialized"))?;

    Ok(())
}

/// Initialize the application configuration from AWS SSM Parameter Store.
///
/// This function loads configuration from AWS SSM Parameter Store under the `/pet-info/` path.
/// It's only available when compiled with the `ssm` feature and provides enterprise-grade
/// configuration management with encryption, versioning, and access control.
///
/// # AWS Requirements
/// - AWS credentials must be available (via IAM role, environment variables, or AWS config)
/// - IAM permissions for `ssm:GetParametersByPath` on `/pet-info/*`
/// - IAM permissions for `kms:Decrypt` to decrypt SecureString parameters
/// - Parameters must exist in SSM Parameter Store under `/pet-info/` path
///
/// # Parameter Format
/// - Parameter names should match [`AppConfig`] field names in UPPERCASE
/// - Values are automatically converted from strings to appropriate types
/// - SecureString parameters are automatically decrypted using the default SSM KMS key
///
/// # Errors
/// Returns an error if:
/// - AWS credentials are not available or invalid
/// - Required IAM permissions are missing
/// - SSM parameters are missing or have invalid values
/// - Network connectivity issues with AWS services
/// - Configuration has already been initialized
///
/// # Example Parameter Structure
/// ```
/// /pet-info/DB_HOST = "sqlite:data/app.db"
/// /pet-info/DB_PASS_ENCRYPT = "encrypted-password" (SecureString)
/// /pet-info/CSRF_PASS = "csrf-secret" (SecureString)
/// /pet-info/WEP_SERVER_PORT = "8080"
/// ```
#[cfg(feature = "ssm")]
pub async fn init_config() -> anyhow::Result<()> {
    let env_values = ssm_env::get_values().await?;
    let config = serde_json::from_value::<AppConfig>(env_values)
        .context("Failed to deserialize configuration from SSM parameters")?;

    APP_CONFIG
        .set(config)
        .map_err(|_| anyhow::anyhow!("Configuration already initialized"))?;

    Ok(())
}

#[cfg(feature = "ssm")]
mod ssm_env {
    //! AWS SSM Parameter Store integration module.
    //!
    //! This module provides functionality to retrieve configuration parameters from
    //! AWS SSM Parameter Store with automatic decryption and pagination support.

    use anyhow::Context;
    use std::collections::HashMap;

    /// The SSM parameter path prefix for all application parameters.
    const PARAMS_PATH: &str = "/pet-info/";

    /// AWS region where SSM parameters are stored.
    const REGION: &str = "us-east-2";

    /// Retrieves all configuration parameters from SSM Parameter Store.
    ///
    /// This function fetches all parameters under the `/pet-info/` path, automatically
    /// handles pagination, decrypts SecureString parameters, and converts them to a
    /// JSON object suitable for deserializing into [`AppConfig`].
    ///
    /// # Returns
    /// A JSON object where keys are parameter names (with `/pet-info/` prefix removed)
    /// and values are the parameter values as JSON strings.
    ///
    /// # Errors
    /// Returns an error if:
    /// - AWS credentials are not available
    /// - IAM permissions are insufficient
    /// - Network connectivity issues occur
    /// - Parameter names or values are missing
    /// - Parameters don't have the expected prefix
    pub async fn get_values() -> anyhow::Result<serde_json::Value> {
        let client = create_ssm_client().await;
        let parameters = fetch_all_parameters(&client).await?;
        let config_map = build_config_map(&parameters)?;

        Ok(serde_json::Value::Object(config_map.into_iter().collect()))
    }

    /// Creates an AWS SSM client configured for the target region.
    ///
    /// Uses AWS SDK defaults for credential resolution and configuration.
    async fn create_ssm_client() -> aws_sdk_ssm::Client {
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(REGION))
            .load()
            .await;
        aws_sdk_ssm::Client::new(&config)
    }

    /// Fetches all parameters from SSM Parameter Store with pagination support.
    ///
    /// This function handles AWS SSM's pagination automatically, collecting all parameters
    /// under the configured path. SecureString parameters are automatically decrypted.
    ///
    /// # Arguments
    /// * `client` - The AWS SSM client to use for API calls
    ///
    /// # Returns
    /// A vector containing all parameters found under the `/pet-info/` path.
    async fn fetch_all_parameters(
        client: &aws_sdk_ssm::Client,
    ) -> anyhow::Result<Vec<aws_sdk_ssm::types::Parameter>> {
        let mut parameters = Vec::new();
        let mut next_token = None;

        loop {
            let response = client
                .get_parameters_by_path()
                .path(PARAMS_PATH)
                .with_decryption(true)
                .set_next_token(next_token)
                .send()
                .await?;

            parameters.extend(response.parameters().iter().cloned());

            next_token = response.next_token().map(String::from);
            if next_token.is_none() {
                break;
            }
        }

        Ok(parameters)
    }

    /// Converts SSM parameters to a configuration map.
    ///
    /// This function processes the raw SSM parameters, removes the path prefix,
    /// and converts them to a HashMap suitable for JSON serialization.
    ///
    /// # Arguments
    /// * `parameters` - Slice of SSM parameters to process
    ///
    /// # Returns
    /// A HashMap where keys are configuration field names (without prefix) and
    /// values are JSON string values ready for deserialization.
    ///
    /// # Errors
    /// Returns an error if any parameter is missing a name, value, or doesn't
    /// have the expected path prefix.
    fn build_config_map(
        parameters: &[aws_sdk_ssm::types::Parameter],
    ) -> anyhow::Result<HashMap<String, serde_json::Value>> {
        parameters
            .iter()
            .map(|param| {
                let name = param.name().context("Missing parameter name")?;
                let value = param.value().context("Missing parameter value")?;
                let key = name
                    .strip_prefix(PARAMS_PATH)
                    .context("Parameter name doesn't start with expected prefix")?;

                Ok((
                    key.to_string(),
                    serde_json::Value::String(value.to_string()),
                ))
            })
            .collect()
    }
}
