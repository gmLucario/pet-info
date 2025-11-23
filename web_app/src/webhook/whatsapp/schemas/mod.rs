//! # WhatsApp Message Schemas
//!
//! This module contains data structures for WhatsApp Business API.
//!
//! - `in`: Incoming webhook schemas (messages received from WhatsApp)
//! - `out`: Outgoing message schemas (messages sent to WhatsApp)

pub mod r#in;
pub mod out;

// Re-export commonly used types
pub use r#in::*;
pub use out::*;
