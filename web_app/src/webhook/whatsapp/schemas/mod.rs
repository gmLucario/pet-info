//! # WhatsApp Message Schemas
//!
//! This module contains data structures for WhatsApp Business API.
//!
//! - `incoming`: Incoming webhook schemas (messages received from WhatsApp)
//! - `outgoing`: Outgoing message schemas (messages sent to WhatsApp)

pub mod incoming;
pub mod outgoing;

// Re-export commonly used types
pub use incoming::*;
pub use outgoing::*;
