//! WhatsApp webhook endpoint handlers
//!
//! This module handles incoming webhook requests from WhatsApp Business API.
//! It implements both the verification endpoint (GET) and the webhook receiver (POST).
//!
//! # Security
//!
//! The POST endpoint verifies webhook authenticity using X-Hub-Signature-256 header.
//! This ensures that requests actually originate from Meta/Facebook.

use super::{handler, schemas, security};
use crate::{
    config,
    front::{AppState, errors},
};
use ntex::{util::Bytes, web};
use serde::Deserialize;

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
/// This endpoint verifies the X-Hub-Signature-256 header to ensure the request
/// originates from Meta/Facebook. Requests with invalid or missing signatures
/// are rejected with a 403 Forbidden response.
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

    // Extract X-Hub-Signature-256 header
    let signature = match req.headers().get("X-Hub-Signature-256") {
        Some(header_value) => match header_value.to_str() {
            Ok(s) => s,
            Err(_) => {
                logfire::warn!("Invalid X-Hub-Signature-256 header: not valid UTF-8");
                return Err(errors::UserError::Unauthorized.into());
            }
        },
        None => {
            logfire::warn!("Missing X-Hub-Signature-256 header in webhook request");
            return Err(errors::UserError::Unauthorized.into());
        }
    };

    // Verify the signature
    if !security::verify_signature(signature, &body, &app_config.whatsapp_app_secret) {
        logfire::warn!("Webhook signature verification failed - rejecting request");
        return Err(errors::UserError::Unauthorized.into());
    }

    // Parse the JSON payload after signature verification
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
}
