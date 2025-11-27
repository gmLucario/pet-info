//! Webhook handlers for external integrations
//!
//! This module contains webhook endpoint handlers for various external services
//! that integrate with the Pet Info application.
//!
//! ## Modules
//!
//! - [`whatsapp`] - WhatsApp Business API webhook handlers
//!
//! ## Future Integrations
//!
//! This module is designed to support multiple webhook integrations:
//! - Mercado Pago payment webhooks
//! - Other messaging platforms
//! - Third-party service integrations

pub mod routes;
pub mod whatsapp;
