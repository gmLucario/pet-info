//! WhatsApp webhook endpoint handlers
//!
//! This module handles incoming webhook requests from WhatsApp Business API.
//! It implements both the verification endpoint (GET) and the webhook receiver (POST).

use super::{handler, schemas};
use crate::front::errors;
use ntex::web;
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
#[web::get("/webhook/whatsapp")]
pub async fn verify(
    query: web::types::Query<VerifyQuery>,
) -> Result<impl web::Responder, web::Error> {
    logfire::info!(
        "Received webhook verification request: mode={mode}, token={token}",
        mode = &query.mode,
        token = &query.verify_token
    );

    // Verify the mode is "subscribe"
    if query.mode != "subscribe" {
        logfire::error!(
            "Invalid mode: expected 'subscribe', got '{mode}'",
            mode = &query.mode
        );
        return Err(errors::UserError::Unauthorized.into());
    }

    // Verify the token matches the configured token
    // TODO: Add WHATSAPP_VERIFY_TOKEN to your config
    let expected_token = std::env::var("WHATSAPP_VERIFY_TOKEN")
        .unwrap_or_else(|_| "your_verify_token_here".to_string());

    if query.verify_token != expected_token {
        logfire::error!("Invalid verify token");
        return Err(errors::UserError::Unauthorized.into());
    }

    logfire::info!("Webhook verification successful");

    // Return the challenge to complete verification
    Ok(web::HttpResponse::Ok()
        .content_type("text/plain")
        .body(query.challenge.clone()))
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
        assert_eq!(query.mode, "subscribe");
        assert_eq!(query.verify_token, "test123");
        assert_eq!(query.challenge, "challenge123");
    }
}
