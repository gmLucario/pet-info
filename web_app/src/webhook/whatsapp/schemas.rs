//! # WhatsApp Webhook Schemas
//!
//! This module contains all data structures for WhatsApp Business API webhooks.
//! These schemas define the JSON payload structure sent by WhatsApp when webhook
//! events occur (incoming messages, status updates, etc.).

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
