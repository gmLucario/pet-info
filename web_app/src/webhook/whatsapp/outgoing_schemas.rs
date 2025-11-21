//! # WhatsApp Outgoing Message Schemas
//!
//! This module contains data structures for sending messages to WhatsApp Business API.
//! These schemas define the JSON payload structure for various message types.

use serde::{Deserialize, Serialize};

/// Text message to send to WhatsApp
#[derive(Debug, Serialize, Deserialize)]
pub struct OutgoingTextMessage {
    /// Messaging product, always "whatsapp"
    pub messaging_product: String,
    /// Recipient's WhatsApp ID (phone number)
    pub to: String,
    /// Message type
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Text content
    pub text: OutgoingTextContent,
}

impl OutgoingTextMessage {
    /// Creates a new text message
    pub fn new(to: String, body: String) -> Self {
        Self {
            messaging_product: "whatsapp".to_string(),
            to,
            msg_type: "text".to_string(),
            text: OutgoingTextContent { body },
        }
    }
}

/// Text content for outgoing messages
#[derive(Debug, Serialize, Deserialize)]
pub struct OutgoingTextContent {
    /// Message body text
    pub body: String,
}

/// Interactive list message to send to WhatsApp
#[derive(Debug, Serialize, Deserialize)]
pub struct OutgoingInteractiveMessage {
    /// Messaging product, always "whatsapp"
    pub messaging_product: String,
    /// Recipient's WhatsApp ID (phone number)
    pub to: String,
    /// Message type, "interactive"
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Interactive content
    pub interactive: InteractiveContent,
}

impl OutgoingInteractiveMessage {
    /// Creates a new interactive list message
    pub fn new_list(
        to: String,
        header: String,
        body: String,
        button_text: String,
        rows: Vec<InteractiveRow>,
    ) -> Self {
        Self {
            messaging_product: "whatsapp".to_string(),
            to,
            msg_type: "interactive".to_string(),
            interactive: InteractiveContent {
                interactive_type: "list".to_string(),
                header: Some(InteractiveHeader {
                    header_type: "text".to_string(),
                    text: header,
                }),
                body: InteractiveBody { text: body },
                action: InteractiveAction {
                    button: button_text,
                    sections: vec![InteractiveSection { title: None, rows }],
                },
            },
        }
    }
}

/// Interactive content structure
#[derive(Debug, Serialize, Deserialize)]
pub struct InteractiveContent {
    /// Type of interactive message (e.g., "list")
    #[serde(rename = "type")]
    pub interactive_type: String,
    /// Optional header
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<InteractiveHeader>,
    /// Body text
    pub body: InteractiveBody,
    /// Action (button and sections)
    pub action: InteractiveAction,
}

/// Interactive message header
#[derive(Debug, Serialize, Deserialize)]
pub struct InteractiveHeader {
    /// Header type (e.g., "text")
    #[serde(rename = "type")]
    pub header_type: String,
    /// Header text
    pub text: String,
}

/// Interactive message body
#[derive(Debug, Serialize, Deserialize)]
pub struct InteractiveBody {
    /// Body text
    pub text: String,
}

/// Interactive action (button and sections)
#[derive(Debug, Serialize, Deserialize)]
pub struct InteractiveAction {
    /// Button text
    pub button: String,
    /// List sections
    pub sections: Vec<InteractiveSection>,
}

/// Interactive section containing rows
#[derive(Debug, Serialize, Deserialize)]
pub struct InteractiveSection {
    /// Optional section title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// List of rows in the section
    pub rows: Vec<InteractiveRow>,
}

/// Interactive row (list item)
#[derive(Debug, Serialize, Deserialize)]
pub struct InteractiveRow {
    /// Unique row ID
    pub id: String,
    /// Row title (displayed to user)
    pub title: String,
    /// Optional row description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl InteractiveRow {
    /// Creates a new interactive row
    pub fn new(id: String, title: String) -> Self {
        Self {
            id,
            title,
            description: None,
        }
    }

    /// Creates a new interactive row with description
    pub fn new_with_description(id: String, title: String, description: String) -> Self {
        Self {
            id,
            title,
            description: Some(description),
        }
    }
}

/// Document message to send to WhatsApp
#[derive(Debug, Serialize, Deserialize)]
pub struct OutgoingDocumentMessage {
    /// Messaging product, always "whatsapp"
    pub messaging_product: String,
    /// Recipient's WhatsApp ID (phone number)
    pub to: String,
    /// Message type, "document"
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Document content
    pub document: DocumentContent,
}

impl OutgoingDocumentMessage {
    /// Creates a new document message with link
    pub fn new_with_link(to: String, link: String, filename: String) -> Self {
        Self {
            messaging_product: "whatsapp".to_string(),
            to,
            msg_type: "document".to_string(),
            document: DocumentContent {
                link: Some(link),
                id: None,
                caption: None,
                filename: Some(filename),
            },
        }
    }

    /// Creates a new document message with media ID
    pub fn new_with_id(to: String, id: String, filename: String) -> Self {
        Self {
            messaging_product: "whatsapp".to_string(),
            to,
            msg_type: "document".to_string(),
            document: DocumentContent {
                link: None,
                id: Some(id),
                caption: None,
                filename: Some(filename),
            },
        }
    }
}

/// Document content
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentContent {
    /// Link to the document (either link or id must be present)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    /// Media ID of the document (either link or id must be present)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Optional caption
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    /// Document filename
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
}

/// Response from WhatsApp API when sending a message
#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppMessageResponse {
    /// Messaging product
    pub messaging_product: String,
    /// Array of contacts (recipients)
    pub contacts: Vec<WhatsAppContact>,
    /// Array of messages sent
    pub messages: Vec<WhatsAppMessageStatus>,
}

/// Contact information in response
#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppContact {
    /// WhatsApp ID of the contact
    pub wa_id: String,
    /// Input phone number
    pub input: String,
}

/// Message status in response
#[derive(Debug, Serialize, Deserialize)]
pub struct WhatsAppMessageStatus {
    /// Message ID
    pub id: String,
}
