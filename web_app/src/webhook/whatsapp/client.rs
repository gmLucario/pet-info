//! # WhatsApp API Client
//!
//! This module provides a client for sending messages to WhatsApp Business API.
//! It handles authentication and message sending for text, interactive, and document messages.

use super::outgoing_schemas::{
    OutgoingDocumentMessage, OutgoingInteractiveMessage, OutgoingTextMessage,
    WhatsAppMessageResponse,
};
use crate::config;
use anyhow::{Context, Result};

/// WhatsApp API client for sending messages
pub struct WhatsAppClient {
    /// HTTP client for making API requests
    client: awc::Client,
    /// WhatsApp Business API endpoint
    endpoint: String,
    /// Authentication token
    auth_token: String,
}

impl WhatsAppClient {
    /// Creates a new WhatsApp client
    pub fn new() -> Result<Self> {
        let app_config = config::APP_CONFIG
            .get()
            .context("failed to get app config")?;

        Ok(Self {
            client: awc::Client::default(),
            endpoint: app_config.whatsapp_send_msg_endpoint(),
            auth_token: app_config.whatsapp_business_auth.clone(),
        })
    }

    /// Sends a text message
    ///
    /// # Arguments
    /// * `to` - Recipient's WhatsApp ID (phone number with country code)
    /// * `body` - Message text
    ///
    /// # Returns
    /// * `Result<WhatsAppMessageResponse>` - Response from WhatsApp API
    pub async fn send_text_message(
        &self,
        to: String,
        body: String,
    ) -> Result<WhatsAppMessageResponse> {
        let message = OutgoingTextMessage::new(to, body);
        self.send_message(&message).await
    }

    /// Sends an interactive list message
    ///
    /// # Arguments
    /// * `message` - Interactive message to send
    ///
    /// # Returns
    /// * `Result<WhatsAppMessageResponse>` - Response from WhatsApp API
    pub async fn send_interactive_message(
        &self,
        message: &OutgoingInteractiveMessage,
    ) -> Result<WhatsAppMessageResponse> {
        self.send_message(message).await
    }

    /// Sends a document message
    ///
    /// # Arguments
    /// * `message` - Document message to send
    ///
    /// # Returns
    /// * `Result<WhatsAppMessageResponse>` - Response from WhatsApp API
    pub async fn send_document_message(
        &self,
        message: &OutgoingDocumentMessage,
    ) -> Result<WhatsAppMessageResponse> {
        self.send_message(message).await
    }

    /// Internal method to send any message type to WhatsApp API
    async fn send_message<T: serde::Serialize>(
        &self,
        message: &T,
    ) -> Result<WhatsAppMessageResponse> {
        let response = self
            .client
            .post(&self.endpoint)
            .insert_header(("Authorization", format!("Bearer {}", self.auth_token)))
            .insert_header(("Content-Type", "application/json"))
            .send_json(message)
            .await
            .context("Failed to send request to WhatsApp API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .body()
                .await
                .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
                .unwrap_or_else(|_| "Unable to read response body".to_string());

            anyhow::bail!(
                "WhatsApp API returned error status {}: {}",
                status,
                body
            );
        }

        let whatsapp_response: WhatsAppMessageResponse = response
            .json()
            .await
            .context("Failed to parse WhatsApp API response")?;

        Ok(whatsapp_response)
    }
}

impl Default for WhatsAppClient {
    fn default() -> Self {
        Self::new().expect("Failed to create WhatsApp client")
    }
}
