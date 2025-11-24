//! WhatsApp webhook integration module
//!
//! This module provides webhook handling for WhatsApp Business API integration.
//! It includes both the HTTP route handlers and the business logic for processing
//! incoming messages and status updates.
//!
//! ## Submodules
//!
//! - [`handler`] - Business logic for processing WhatsApp webhook events
//! - [`routes`] - HTTP endpoint handlers for WhatsApp webhooks
//! - [`schemas`] - Data structures for WhatsApp webhook payloads (incoming and outgoing)
//! - [`client`] - WhatsApp API client for sending messages
//! - [`security`] - Security utilities for webhook signature verification

pub mod client;
pub mod handler;
pub mod routes;
pub mod schemas;
pub mod security;

// Re-export commonly used items for convenience
pub use routes::{receive, verify};
