//! # API Module
//!
//! This module contains all the business logic and data processing functions
//! for the Pet Info application. Each submodule handles a specific domain
//! of functionality.
//!
//! ## Modules
//!
//! - [`passes`] - Apple Wallet pass generation and handling
//! - [`payment`] - Payment processing and billing operations
//! - [`pdf_handler`] - PDF generation and report handling
//! - [`pet`] - Pet management, profiles, and health records
//! - [`reminder`] - Notification and reminder systems
//! - [`user`] - User management and authentication

pub mod passes;
pub mod payment;
pub mod pdf_handler;
pub mod pet;
pub mod reminder;
pub mod user;
