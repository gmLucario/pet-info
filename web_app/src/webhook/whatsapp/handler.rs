//! # WhatsApp Webhook Handler
//!
//! This module handles incoming webhook events from WhatsApp Business API.
//! It processes incoming messages, status updates, and other webhook events.

use super::{
    client::WhatsAppClient,
    schemas::{InteractiveRow, Message, OutgoingDocumentMessage, OutgoingInteractiveMessage, Status, WebhookPayload},
};
use crate::repo;
use anyhow::{Context, Result};

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
    statuses
}

/// Sends pet information to a WhatsApp user
///
/// Sends a text message listing all registered pets, followed by an interactive
/// list message for each pet with options for report, QR, and card.
///
/// # Arguments
///
/// * `client` - WhatsApp API client
/// * `to` - Recipient's WhatsApp ID (phone number)
/// * `user_id` - Database ID of the user
/// * `repo` - Repository for database access
async fn send_pet_info_to_user(
    client: &WhatsAppClient,
    to: &str,
    user_id: i64,
    repo: &repo::ImplAppRepo,
) -> Result<()> {
    let _span = logfire::span!("send_pet_info_to_user").entered();

    // Get all pets for the user
    let pets = repo.get_all_pets_user_id(user_id).await?;

    if pets.is_empty() {
        client
            .send_text_message(
                to.to_string(),
                "No tienes mascotas registradas en Pet-Info.".to_string(),
            )
            .await?;
        return Ok(());
    }

    // Send initial text message
    client
        .send_text_message(
            to.to_string(),
            "Tus mascotas registradas en Pet-Info son:".to_string(),
        )
        .await?;

    // Send interactive list message for each pet
    for pet in pets {
        let pet_name = pet.pet_name.clone();
        let external_id = pet.external_id;

        let rows = vec![
            InteractiveRow::new(
                format!("reporte:{}", external_id),
                format!("reporte: [{}]", pet_name),
            ),
            InteractiveRow::new(format!("qr:{}", external_id), format!("qr: [{}]", pet_name)),
            InteractiveRow::new(
                format!("tarjeta:{}", external_id),
                format!("tarjeta: [{}]", pet_name),
            ),
        ];

        let message = OutgoingInteractiveMessage::new_list(
            to.to_string(),
            pet_name.clone(),
            format!(
                "Su perfil público es: https://pet-info.link/info/{}",
                external_id
            ),
            "opciones".to_string(),
            rows,
        );

        client
            .send_interactive_message(&message)
            .await
            .with_context(|| format!("Failed to send interactive message for pet {}", pet_name))?;
    }

    Ok(())
}

/// Handles interactive button responses from users
///
/// Processes user selections from interactive list messages and sends appropriate responses.
///
/// # Arguments
///
/// * `client` - WhatsApp API client
/// * `message` - The message containing the interactive response
/// * `repo` - Repository for database access
async fn handle_interactive_response(
    client: &WhatsAppClient,
    message: &Message,
    repo: &repo::ImplAppRepo,
) -> Result<()> {
    let list_reply = message
        .interactive
        .as_ref()
        .context("No interactive data in message")?
        .list_reply
        .as_ref()
        .context("No list reply in interactive message")?;

    let parts: Vec<&str> = list_reply.id.split(':').collect();
    if parts.len() != 2 {
        logfire::warn!(
            "Invalid interactive response ID format: {id}",
            id = &list_reply.id
        );
        return Ok(());
    }

    let action = parts[0];
    let external_id_str = parts[1];

    let external_id = uuid::Uuid::parse_str(external_id_str)
        .with_context(|| format!("Invalid UUID in interactive response: {}", external_id_str))?;

    match action {
        "reporte" => {
            // Generate PDF report bytes
            logfire::info!(
                "Generating PDF report for pet external_id {id}",
                id = external_id.to_string()
            );

            let pet = repo.get_pet_by_external_id(external_id).await?;
            let pdf_bytes =
                crate::api::pet::generate_pdf_report_bytes(pet.id, pet.user_app_id, repo).await?;

            let pet = repo.get_pet_by_external_id(external_id).await?;

            let filename = format!("reporte_{}.pdf", pet.pet_name);

            // Upload PDF to WhatsApp and get media ID
            logfire::info!(
                "Uploading PDF to WhatsApp for pet {name}",
                name = &pet.pet_name
            );

            let media_id = client
                .upload_media(pdf_bytes, "application/pdf", &filename)
                .await?;

            let pet_name_for_log = pet.pet_name.clone();

            // Send document message with media ID
            let document_message =
                OutgoingDocumentMessage::new_with_id(message.from.clone(), media_id, filename);

            client.send_document_message(&document_message).await?;

            logfire::info!("Sent PDF report for pet {name}", name = &pet_name_for_log);
        }
        "qr" => {
            // Send QR code link
            client
                .send_text_message(
                    message.from.clone(),
                    format!(
                        "Código QR: https://pet-info.link/pet/qr_code/{}",
                        external_id
                    ),
                )
                .await?;

            logfire::info!(
                "Sent QR code link for pet external_id {id}",
                id = external_id.to_string()
            );
        }
        "tarjeta" => {
            // Send Apple Wallet pass link
            client
                .send_text_message(
                    message.from.clone(),
                    format!(
                        "Tarjeta digital: https://pet-info.link/pet/pass/{}",
                        external_id
                    ),
                )
                .await?;

            logfire::info!(
                "Sent pass link for pet external_id {id}",
                id = external_id.to_string()
            );
        }
        _ => {
            logfire::warn!(
                "Unknown action in interactive response: {action}",
                action = action.to_string()
            );
        }
    }

    Ok(())
}

/// Handles incoming messages from users
///
/// This function processes each message and determines the appropriate response.
/// Add your business logic here to handle different message types and content.
///
/// # Arguments
///
/// * `message` - The message to handle
/// * `client` - WhatsApp API client for sending messages
/// * `repo` - Repository for database access
///
/// # Returns
///
/// Result indicating success or failure
pub async fn handle_user_message(
    message: &Message,
    client: &WhatsAppClient,
    repo: &repo::ImplAppRepo,
) -> Result<()> {
    let _span = logfire::span!("handle_user_message").entered();

    match message.msg_type.as_str() {
        "text" => {
            if message.text.is_some() {
                logfire::info!("Received text message from {from}", from = &message.from);

                // Look up user by phone number
                let user = repo.get_user_app_by_phone(&message.from).await?;

                match user {
                    Some(user) => {
                        // Send pet information
                        send_pet_info_to_user(&client, &message.from, user.id, repo).await?;
                    }
                    None => {
                        // User not found
                        client
                            .send_text_message(
                                message.from.clone(),
                                "No se encontró una cuenta asociada a este número de teléfono. \
                                 Por favor, regístrala en tu perfil en https://pet-info.link"
                                    .to_string(),
                            )
                            .await?;
                    }
                }
            }
        }
        "interactive" => {
            logfire::info!(
                "Received interactive response from {from}",
                from = &message.from
            );
            handle_interactive_response(&client, message, repo).await?;
        }
        "image" => {
            if message.image.is_some() {
                logfire::info!("Received image");
                // TODO: Handle image uploads (e.g., pet photos)
            }
        }
        "location" => {
            if message.location.is_some() {
                logfire::info!("Received location");
                // TODO: Handle location sharing (e.g., lost pet location)
            }
        }
        "document" => {
            if message.document.is_some() {
                logfire::info!("Received document");
                // TODO: Handle document uploads
            }
        }
        _ => {
            logfire::warn!(
                "Unsupported message type received: {type}",
                r#type = &message.msg_type
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
pub async fn handle_message_status(_status: &Status) -> Result<()> {
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
/// * `client` - WhatsApp API client for sending messages
/// * `repo` - Repository for database access
///
/// # Returns
///
/// Result indicating success or failure
pub async fn process_webhook(
    payload: WebhookPayload,
    client: &WhatsAppClient,
    repo: &repo::ImplAppRepo,
) -> Result<()> {
    // Process incoming messages
    let messages = process_webhook_messages(&payload);
    for message in messages {
        if let Err(e) = handle_user_message(message, client, repo).await {
            logfire::error!("Failed to handle message: {error}", error = e.to_string());
        }
    }

    // Process status updates
    let statuses = process_webhook_statuses(&payload);
    for status in statuses {
        if let Err(e) = handle_message_status(status).await {
            logfire::error!("Failed to handle status: {error}", error = e.to_string());
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
                            interactive: None,
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
