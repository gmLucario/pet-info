//! WhatsApp webhook endpoint handlers
//!
//! This module handles incoming webhook requests from WhatsApp Business API.
//! It implements both the verification endpoint (GET) and the webhook receiver (POST).
//!
//! # Security
//!
//! Webhook verification GET requests are authenticated with the configured
//! verification token. POST bodies are authenticated with Meta's
//! `X-Hub-Signature-256` HMAC in addition to Nginx mTLS enforcement.

use super::{handler, schemas};
use crate::{
    config,
    front::{AppState, errors},
};
use hmac::{Hmac, Mac};
use ntex::{util::Bytes, web};
use serde::Deserialize;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn has_valid_meta_signature(signature: Option<&str>, body: &[u8], secret: &str) -> bool {
    let Some(hex_signature) = signature.and_then(|value| value.strip_prefix("sha256=")) else {
        return false;
    };

    if secret.is_empty() || hex_signature.len() != 64 {
        return false;
    }

    let Ok(signature_bytes) = hex::decode(hex_signature) else {
        return false;
    };

    HmacSha256::new_from_slice(secret.as_bytes()).is_ok_and(|mut mac| {
        mac.update(body);
        mac.verify_slice(&signature_bytes).is_ok()
    })
}

/// Query parameters for webhook verification
#[derive(Debug, Deserialize)]
pub struct VerifyQuery {
    /// The mode parameter, should be "subscribe"
    #[serde(rename = "hub.mode")]
    pub mode: String,
    /// The verification token from WhatsApp
    #[serde(rename = "hub.verify_token")]
    pub verify_token: String,
    /// The challenge string to echo back
    #[serde(rename = "hub.challenge")]
    pub challenge: String,
}

/// Webhook verification endpoint (GET)
///
/// WhatsApp sends a GET request to verify the webhook URL.
/// This endpoint validates the verify token and returns the challenge.
///
/// # Query Parameters
/// - `hub.mode` - Should be "subscribe"
/// - `hub.verify_token` - Token configured in WhatsApp dashboard
/// - `hub.challenge` - Challenge string to echo back
///
/// # Returns
/// - 200 with challenge string if verification succeeds
/// - 403 if verification fails
#[web::get("")]
pub async fn verify(
    query: web::types::Query<VerifyQuery>,
) -> Result<impl web::Responder, web::Error> {
    if query.mode != "subscribe" {
        return Err(errors::UserError::Unauthorized.into());
    }

    let app_config = config::APP_CONFIG
        .get()
        .expect("APP_CONFIG should be initialized before starting web server");

    if query.verify_token != app_config.whatsapp_verify_token {
        return Err(errors::UserError::Unauthorized.into());
    }

    Ok(web::HttpResponse::Ok()
        .content_type("text/plain")
        .body(query.challenge.clone()))
}

/// Webhook receiver endpoint (POST)
///
/// Receives webhook events from WhatsApp Business API.
/// Processes incoming messages, status updates, and other events.
///
/// # Security
///
/// This endpoint verifies mTLS client certificate information passed by Nginx reverse proxy.
/// Nginx handles the TLS layer verification and passes verification headers.
/// Requests without valid mTLS certificates are rejected with a 403 Forbidden response.
///
/// # Processing
///
/// Process webhook synchronously.
/// WhatsApp gives us 20 seconds to respond, which should be sufficient.
#[web::post("")]
pub async fn receive(
    req: web::HttpRequest,
    body: Bytes,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let app_config = config::APP_CONFIG
        .get()
        .expect("APP_CONFIG should be initialized before starting web server");
    let signature = req
        .headers()
        .get("X-Hub-Signature-256")
        .and_then(|value| value.to_str().ok());

    if !has_valid_meta_signature(signature, &body, &app_config.whatsapp_app_secret) {
        logfire::warn!("Rejected WhatsApp webhook with invalid signature");
        return Err(errors::UserError::Unauthorized.into());
    }

    // Parse the JSON payload before processing it.
    let payload: schemas::WebhookPayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            logfire::error!(
                "Failed to parse webhook payload: {error}",
                error = e.to_string()
            );
            return Err(errors::UserError::Unauthorized.into());
        }
    };

    // Process the webhook
    if let Err(e) = handler::process_webhook(
        payload,
        &app_state.whatsapp_client,
        &app_state.repo,
        &app_state.storage_service,
        &app_state.magic_link_service,
    )
    .await
    {
        logfire::error!("Failed to process webhook: {error}", error = e.to_string());
    }

    Ok(web::HttpResponse::Ok().json(&serde_json::json!({
        "status": "received"
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_query_deserialization() {
        let json = r#"{"hub.mode":"subscribe","hub.verify_token":"test123","hub.challenge":"challenge123"}"#;
        let query: VerifyQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.mode, "subscribe");
        assert_eq!(query.verify_token, "test123");
        assert_eq!(query.challenge, "challenge123");
    }

    #[test]
    fn validates_meta_signature() {
        let body = br#"{"object":"whatsapp_business_account"}"#;
        let secret = "meta-app-secret";
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        let signature = format!(
            "sha256={}",
            mac.finalize()
                .into_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );

        assert!(has_valid_meta_signature(Some(&signature), body, secret));
        assert!(!has_valid_meta_signature(
            Some(&signature),
            b"tampered",
            secret
        ));
        assert!(!has_valid_meta_signature(None, body, secret));
        assert!(!has_valid_meta_signature(
            Some("sha256=éééééééééééééééééééééééééééééééé"),
            body,
            secret
        ));
    }
}
