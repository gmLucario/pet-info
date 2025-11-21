//! # WhatsApp Webhook Handler
//!
//! This module handles incoming webhook events from WhatsApp Business API.
//! It processes incoming messages, status updates, and other webhook events.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Root webhook payload from WhatsApp
#[derive(Debug, Deserialize, Serialize)]
pub struct WebhookPayload {
    /// The object type, typically "whatsapp_business_account"
    pub object: String,
    /// Array of entry objects containing the actual data
    pub entry: Vec<Entry>,
}

/// Entry object containing changes and metadata
#[derive(Debug, Deserialize, Serialize)]
pub struct Entry {
    /// Business Account ID
    pub id: String,
    /// Array of changes that occurred
    pub changes: Vec<Change>,
}

/// Change object containing the actual webhook data
#[derive(Debug, Deserialize, Serialize)]
pub struct Change {
    /// The field that changed (e.g., "messages")
    pub field: String,
    /// The value containing the actual data
    pub value: Value,
}

/// Value object containing messages and metadata
#[derive(Debug, Deserialize, Serialize)]
pub struct Value {
    /// Messaging product (e.g., "whatsapp")
    pub messaging_product: String,
    /// Metadata about the phone number
    pub metadata: Metadata,
    /// Array of contacts (senders)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contacts: Option<Vec<Contact>>,
    /// Array of messages received
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<Message>>,
    /// Array of statuses (for sent messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statuses: Option<Vec<Status>>,
}

/// Metadata about the WhatsApp Business phone number
#[derive(Debug, Deserialize, Serialize)]
pub struct Metadata {
    /// Display name of the business phone number
    pub display_phone_number: String,
    /// Phone number ID
    pub phone_number_id: String,
}

/// Contact information for the message sender
#[derive(Debug, Deserialize, Serialize)]
pub struct Contact {
    /// Profile information
    pub profile: Profile,
    /// WhatsApp ID (phone number)
    pub wa_id: String,
}

/// Profile information
#[derive(Debug, Deserialize, Serialize)]
pub struct Profile {
    /// Display name of the contact
    pub name: String,
}

/// Message object
#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    /// Sender's WhatsApp ID (phone number)
    pub from: String,
    /// Message ID
    pub id: String,
    /// Timestamp of the message
    pub timestamp: String,
    /// Message type (text, image, video, document, etc.)
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Text message content (if type is "text")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextMessage>,
    /// Image message content (if type is "image")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<MediaMessage>,
    /// Video message content (if type is "video")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<MediaMessage>,
    /// Document message content (if type is "document")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<MediaMessage>,
    /// Audio message content (if type is "audio")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<MediaMessage>,
    /// Location message content (if type is "location")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<LocationMessage>,
    /// Context (if this is a reply to another message)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Context>,
}

/// Text message content
#[derive(Debug, Deserialize, Serialize)]
pub struct TextMessage {
    /// The text body of the message
    pub body: String,
}

/// Media message content (image, video, document, audio)
#[derive(Debug, Deserialize, Serialize)]
pub struct MediaMessage {
    /// Media ID
    pub id: String,
    /// MIME type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// SHA256 hash of the media
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// Caption (for image, video, document)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
}

/// Location message content
#[derive(Debug, Deserialize, Serialize)]
pub struct LocationMessage {
    /// Latitude
    pub latitude: f64,
    /// Longitude
    pub longitude: f64,
    /// Name of the location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Address of the location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}

/// Context for reply messages
#[derive(Debug, Deserialize, Serialize)]
pub struct Context {
    /// ID of the message being replied to
    pub from: String,
    /// Message ID being referenced
    pub id: String,
}

/// Status update for sent messages
#[derive(Debug, Deserialize, Serialize)]
pub struct Status {
    /// Message ID
    pub id: String,
    /// Status (sent, delivered, read, failed)
    pub status: String,
    /// Timestamp
    pub timestamp: String,
    /// Recipient ID
    pub recipient_id: String,
}

/// Processes incoming WhatsApp webhook messages
///
/// Extracts and processes all messages from the webhook payload.
/// Messages are filtered to only include "messages" field changes.
///
/// # Arguments
///
/// * `payload` - The webhook payload from WhatsApp
///
/// # Returns
///
/// A vector of processed messages
pub fn process_webhook_messages(payload: &WebhookPayload) -> Vec<&Message> {
    let messages = payload
        .entry
        .iter()
        .flat_map(|entry| &entry.changes)
        .filter(|change| change.field == "messages")
        .filter_map(|change| change.value.messages.as_ref())
        .flatten()
        .collect::<Vec<_>>();

    logfire::info!(
        "Processed {count} messages from webhook",
        count = messages.len()
    );
    messages
}

/// Processes incoming WhatsApp webhook statuses
///
/// Extracts and processes all status updates from the webhook payload.
///
/// # Arguments
///
/// * `payload` - The webhook payload from WhatsApp
///
/// # Returns
///
/// A vector of processed statuses
pub fn process_webhook_statuses(payload: &WebhookPayload) -> Vec<&Status> {
    let statuses = payload
        .entry
        .iter()
        .flat_map(|entry| &entry.changes)
        .filter(|change| change.field == "messages")
        .filter_map(|change| change.value.statuses.as_ref())
        .flatten()
        .collect::<Vec<_>>();

    logfire::info!(
        "Processed {count} statuses from webhook",
        count = statuses.len()
    );
    statuses
}

/// Handles incoming messages from users
///
/// This function processes each message and determines the appropriate response.
/// Add your business logic here to handle different message types and content.
///
/// # Arguments
///
/// * `message` - The message to handle
///
/// # Returns
///
/// Result indicating success or failure
pub async fn handle_user_message(message: &Message) -> Result<()> {
    logfire::info!(
        "Handling message from {from}: type={msg_type}",
        from = &message.from,
        msg_type = &message.msg_type
    );

    match message.msg_type.as_str() {
        "text" => {
            if let Some(text) = &message.text {
                logfire::info!("Received text message: {body}", body = &text.body);
                // TODO: Add your message handling logic here
                // Example: Parse commands, look up pet info, send responses
            }
        }
        "image" => {
            if let Some(image) = &message.image {
                logfire::info!("Received image: id={id}", id = &image.id);
                // TODO: Handle image uploads (e.g., pet photos)
            }
        }
        "location" => {
            if let Some(location) = &message.location {
                logfire::info!(
                    "Received location: lat={lat}, lon={lon}",
                    lat = location.latitude,
                    lon = location.longitude
                );
                // TODO: Handle location sharing (e.g., lost pet location)
            }
        }
        "document" => {
            if let Some(document) = &message.document {
                logfire::info!("Received document: id={id}", id = &document.id);
                // TODO: Handle document uploads
            }
        }
        _ => {
            logfire::warn!(
                "Unsupported message type: {msg_type}",
                msg_type = &message.msg_type
            );
        }
    }

    Ok(())
}

/// Handles status updates for sent messages
///
/// This function processes status updates to track message delivery.
///
/// # Arguments
///
/// * `status` - The status update to handle
///
/// # Returns
///
/// Result indicating success or failure
pub async fn handle_message_status(status: &Status) -> Result<()> {
    logfire::info!(
        "Message {id} status: {msg_status} for recipient {recipient}",
        id = &status.id,
        msg_status = &status.status,
        recipient = &status.recipient_id
    );

    // TODO: Update your database with delivery status if needed

    Ok(())
}

/// Main webhook processor
///
/// Processes the complete webhook payload, handling both messages and statuses.
///
/// # Arguments
///
/// * `payload` - The webhook payload from WhatsApp
///
/// # Returns
///
/// Result indicating success or failure
pub async fn process_webhook(payload: WebhookPayload) -> Result<()> {
    logfire::info!(
        "Processing webhook with {entries} entries",
        entries = payload.entry.len()
    );

    // Process incoming messages
    let messages = process_webhook_messages(&payload);
    for message in messages {
        if let Err(e) = handle_user_message(message).await {
            logfire::error!(
                "Failed to handle message {id}: {error}",
                id = &message.id,
                error = e.to_string()
            );
        }
    }

    // Process status updates
    let statuses = process_webhook_statuses(&payload);
    for status in statuses {
        if let Err(e) = handle_message_status(status).await {
            logfire::error!(
                "Failed to handle status {id}: {error}",
                id = &status.id,
                error = e.to_string()
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_webhook_messages() {
        let payload = WebhookPayload {
            object: "whatsapp_business_account".to_string(),
            entry: vec![Entry {
                id: "123456".to_string(),
                changes: vec![Change {
                    field: "messages".to_string(),
                    value: Value {
                        messaging_product: "whatsapp".to_string(),
                        metadata: Metadata {
                            display_phone_number: "+1234567890".to_string(),
                            phone_number_id: "phone123".to_string(),
                        },
                        contacts: None,
                        messages: Some(vec![Message {
                            from: "+9876543210".to_string(),
                            id: "msg123".to_string(),
                            timestamp: "1234567890".to_string(),
                            msg_type: "text".to_string(),
                            text: Some(TextMessage {
                                body: "Hello".to_string(),
                            }),
                            image: None,
                            video: None,
                            document: None,
                            audio: None,
                            location: None,
                            context: None,
                        }]),
                        statuses: None,
                    },
                }],
            }],
        };

        let messages = process_webhook_messages(&payload);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].from, "+9876543210");
    }
}
