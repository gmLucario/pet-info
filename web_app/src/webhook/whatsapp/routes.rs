//! WhatsApp webhook endpoint handlers
//!
//! This module handles incoming webhook requests from WhatsApp Business API.
//! It implements both the verification endpoint (GET) and the webhook receiver (POST).

use super::{handler, schemas};
use crate::{config, front::errors};
use ntex::web;
use serde::Deserialize;

/// Query parameters for webhook verification
#[derive(Debug, Deserialize)]
pub struct VerifyQuery {
    /// The mode parameter, should be "subscribe"
    #[serde(rename = "hub.mode")]
    pub mode: Option<String>,
    /// The verification token from WhatsApp
    #[serde(rename = "hub.verify_token")]
    pub verify_token: Option<String>,
    /// The challenge string to echo back
    #[serde(rename = "hub.challenge")]
    pub challenge: Option<String>,
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
#[web::get("/webhook/whatsapp")]
pub async fn verify(
    query: web::types::Query<VerifyQuery>,
) -> Result<impl web::Responder, web::Error> {
    // Extract required query parameters
    let mode = match &query.mode {
        Some(m) => m,
        None => {
            logfire::warn!("Webhook verification request missing hub.mode parameter");
            return Err(errors::UserError::FormInputValueError(
                "Missing required parameter: hub.mode".to_string()
            ).into());
        }
    };

    let verify_token = match &query.verify_token {
        Some(t) => t,
        None => {
            logfire::warn!("Webhook verification request missing hub.verify_token parameter");
            return Err(errors::UserError::FormInputValueError(
                "Missing required parameter: hub.verify_token".to_string()
            ).into());
        }
    };

    let challenge = match &query.challenge {
        Some(c) => c,
        None => {
            logfire::warn!("Webhook verification request missing hub.challenge parameter");
            return Err(errors::UserError::FormInputValueError(
                "Missing required parameter: hub.challenge".to_string()
            ).into());
        }
    };

    logfire::info!(
        "Received webhook verification request: mode={mode}, token={token}",
        mode = mode,
        token = verify_token
    );

    // Verify the mode is "subscribe"
    if mode != "subscribe" {
        logfire::error!(
            "Invalid mode: expected 'subscribe', got '{mode}'",
            mode = mode
        );
        return Err(errors::UserError::Unauthorized.into());
    }

    // Verify the token matches the configured token
    let app_config = config::APP_CONFIG
        .get()
        .expect("APP_CONFIG should be initialized before starting web server");

    if verify_token != &app_config.whatsapp_verify_token {
        logfire::error!("Invalid verify token");
        return Err(errors::UserError::Unauthorized.into());
    }

    logfire::info!("Webhook verification successful");

    // Return the challenge to complete verification
    Ok(web::HttpResponse::Ok()
        .content_type("text/plain")
        .body(challenge.clone()))
}

/// Webhook receiver endpoint (POST)
///
/// Receives webhook events from WhatsApp Business API.
/// Processes incoming messages, status updates, and other events.
///
/// # Request Body
/// JSON payload containing webhook data from WhatsApp
///
/// # Returns
/// - 200 OK if processing succeeds
/// - 500 if processing fails
#[web::post("/webhook/whatsapp")]
pub async fn receive(
    payload: web::types::Json<schemas::WebhookPayload>,
) -> Result<impl web::Responder, web::Error> {
    let _span = logfire::span!("whatsapp_webhook").entered();

    logfire::info!(
        "Received webhook: object={object}, entries={entries}",
        object = &payload.object,
        entries = payload.entry.len()
    );

    // Process the webhook asynchronously
    // We return 200 immediately to acknowledge receipt
    let payload_clone = payload.into_inner();

    // Spawn a task to process the webhook in the background
    ntex::rt::spawn(async move {
        if let Err(e) = handler::process_webhook(payload_clone).await {
            logfire::error!("Failed to process webhook: {error}", error = e.to_string());
        }
    });

    // Return 200 OK immediately to acknowledge receipt
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
        assert_eq!(query.mode, Some("subscribe".to_string()));
        assert_eq!(query.verify_token, Some("test123".to_string()));
        assert_eq!(query.challenge, Some("challenge123".to_string()));
    }

    #[test]
    fn test_verify_query_deserialization_empty() {
        let json = r#"{}"#;
        let query: VerifyQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.mode, None);
        assert_eq!(query.verify_token, None);
        assert_eq!(query.challenge, None);
    }
}
