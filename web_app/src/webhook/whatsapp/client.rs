//! # WhatsApp API Client
//!
//! This module provides a client for sending messages to WhatsApp Business API.
//! It handles authentication and message sending for text, interactive, and document messages.

use super::schemas::{
    OutgoingDocumentMessage, OutgoingInteractiveMessage, OutgoingTextMessage,
    WhatsAppMessageResponse,
};
use crate::config;
use anyhow::{Context, Result};

/// Response from WhatsApp media upload API
#[derive(Debug, serde::Deserialize)]
pub struct MediaUploadResponse {
    /// Uploaded media ID
    pub id: String,
}

/// Typing indicator action
#[derive(Debug, serde::Serialize)]
struct TypingIndicator {
    messaging_product: String,
    recipient_type: String,
    to: String,
    #[serde(rename = "type")]
    typing_action: String,
}

/// WhatsApp API client for sending messages and uploading media
#[derive(Clone)]
pub struct WhatsAppClient {
    /// HTTP client for making API requests
    client: reqwest::Client,
    /// WhatsApp Business API endpoint for sending messages
    endpoint: String,
    /// WhatsApp Business phone number ID
    phone_number_id: u64,
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
            client: reqwest::Client::new(),
            endpoint: app_config.whatsapp_send_msg_endpoint(),
            phone_number_id: app_config.whatsapp_business_phone_number_id,
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

    /// Sends a template message (for OTP verification, notifications, etc.)
    ///
    /// Template messages are pre-approved message formats used for sending
    /// notifications, OTPs, and other structured messages.
    ///
    /// # Arguments
    /// * `payload` - Template message payload as JSON
    ///
    /// # Returns
    /// * `Result<WhatsAppMessageResponse>` - Response from WhatsApp API
    ///
    /// # Example
    /// ```no_run
    /// let payload = json!({
    ///     "messaging_product": "whatsapp",
    ///     "to": "+1234567890",
    ///     "type": "template",
    ///     "template": { ... }
    /// });
    /// client.send_template_message(payload).await?;
    /// ```
    pub async fn send_template_message(
        &self,
        payload: serde_json::Value,
    ) -> Result<WhatsAppMessageResponse> {
        self.send_message(&payload).await
    }

    /// Sends a typing indicator "on" to show the user that a message is being prepared
    ///
    /// This creates a visual feedback in the WhatsApp chat showing three dots
    /// to indicate that a response is being typed.
    ///
    /// # Arguments
    /// * `to` - Recipient's WhatsApp ID (phone number with country code)
    ///
    /// # Returns
    /// * `Result<WhatsAppMessageResponse>` - Response from WhatsApp API
    ///
    /// # Example
    /// ```no_run
    /// client.send_typing_on("+1234567890".to_string()).await?;
    /// ```
    pub async fn send_typing_on(&self, to: String) -> Result<WhatsAppMessageResponse> {
        let indicator = TypingIndicator {
            messaging_product: "whatsapp".to_string(),
            recipient_type: "individual".to_string(),
            to,
            typing_action: "typing_on".to_string(),
        };
        self.send_message(&indicator).await
    }

    /// Uploads media (document, image, etc.) to WhatsApp and returns media ID
    ///
    /// Uploads file bytes to WhatsApp's media upload API and returns a media ID
    /// that can be used in subsequent message sends.
    ///
    /// # Arguments
    /// * `file_bytes` - The file content as bytes
    /// * `mime_type` - MIME type of the file (e.g., "application/pdf", "image/jpeg")
    /// * `filename` - Name of the file
    ///
    /// # Returns
    /// * `Result<String>` - Media ID that can be used to send the document
    ///
    /// # Example
    /// ```no_run
    /// let media_id = client.upload_media(pdf_bytes, "application/pdf", "report.pdf").await?;
    /// ```
    pub async fn upload_media(
        &self,
        file_bytes: Vec<u8>,
        mime_type: &str,
        filename: &str,
    ) -> Result<String> {
        let upload_endpoint = format!(
            "https://graph.facebook.com/v22.0/{}/media",
            self.phone_number_id
        );

        // Create multipart form using reqwest
        let file_part = reqwest::multipart::Part::bytes(file_bytes)
            .file_name(filename.to_string())
            .mime_str(mime_type)?;

        let form = reqwest::multipart::Form::new()
            .text("messaging_product", "whatsapp")
            .part("file", file_part);

        let response = self
            .client
            .post(&upload_endpoint)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .multipart(form)
            .send()
            .await
            .context("Failed to upload media to WhatsApp API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read response body".to_string());

            anyhow::bail!(
                "WhatsApp media upload returned error status {}: {}",
                status,
                body
            );
        }

        let upload_response: MediaUploadResponse = response
            .json()
            .await
            .context("Failed to parse WhatsApp media upload response")?;

        Ok(upload_response.id)
    }

    /// Internal method to send any message type to WhatsApp API
    async fn send_message<T: serde::Serialize>(
        &self,
        message: &T,
    ) -> Result<WhatsAppMessageResponse> {
        let response = self
            .client
            .post(&self.endpoint)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .header("Content-Type", "application/json")
            .json(message)
            .send()
            .await
            .context("Failed to send request to WhatsApp API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read response body".to_string());

            anyhow::bail!("WhatsApp API returned error status {}: {}", status, body);
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
