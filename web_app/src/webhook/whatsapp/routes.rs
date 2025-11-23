//! WhatsApp webhook endpoint handlers
//!
//! This module handles incoming webhook requests from WhatsApp Business API.
//! It implements both the verification endpoint (GET) and the webhook receiver (POST).

use super::{handler, schemas};
use crate::{
    config,
    front::{AppState, errors},
};
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
#[web::get("")]
pub async fn verify(
    query: web::types::Query<VerifyQuery>,
) -> Result<impl web::Responder, web::Error> {
    logfire::info!("Received webhook verification request");

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
/// # Request Body
/// JSON payload containing webhook data from WhatsApp
///
/// # Returns
/// - 200 OK if processing succeeds
/// - 500 if processing fails
#[web::post("")]
pub async fn receive(
    payload: web::types::Json<schemas::WebhookPayload>,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let payload = payload.into_inner();

    // Process webhook synchronously
    // WhatsApp gives us 20 seconds to respond, which should be sufficient
    if let Err(e) = handler::process_webhook(payload, &app_state.whatsapp_client, &app_state.repo).await {
        logfire::error!("Failed to process webhook: {error}", error = e.to_string());
        // Still return 200 to acknowledge receipt even if processing fails
        // This prevents WhatsApp from retrying failed messages
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
