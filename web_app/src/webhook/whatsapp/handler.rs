//! # WhatsApp Webhook Handler
//!
//! This module handles incoming webhook events from WhatsApp Business API.
//! It processes incoming messages, status updates, and other webhook events.

use super::schemas::{Message, Status, WebhookPayload};
use anyhow::Result;

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
    use crate::webhook::whatsapp::schemas::*;

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
