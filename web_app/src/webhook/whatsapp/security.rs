//! Security utilities for WhatsApp webhook verification
//!
//! This module provides signature verification for incoming WhatsApp webhook requests
//! using the X-Hub-Signature-256 header. This ensures that requests actually originate
//! from Meta/Facebook and haven't been tampered with.
//!
//! # Security Background
//!
//! Meta signs all webhook payloads with HMAC-SHA256 using your app's secret key.
//! The signature is included in the `X-Hub-Signature-256` header with the format:
//! `sha256=<hex_signature>`
//!
//! To verify authenticity:
//! 1. Extract the signature from the X-Hub-Signature-256 header
//! 2. Compute HMAC-SHA256 of the raw request body using your app secret
//! 3. Compare the computed signature with the received signature
//! 4. Only process the request if signatures match
//!
//! # Important Notes
//!
//! - The signature MUST be computed on the raw request body bytes, not parsed JSON
//! - The comparison must be constant-time to prevent timing attacks
//! - The header format is `sha256=<signature>` (lowercase)

use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

/// Verifies the X-Hub-Signature-256 header against the request payload
///
/// # Arguments
///
/// * `signature_header` - The value of the X-Hub-Signature-256 header (e.g., "sha256=abc123...")
/// * `payload` - The raw request body bytes
/// * `app_secret` - Your WhatsApp/Facebook app secret
///
/// # Returns
///
/// * `true` if the signature is valid
/// * `false` if the signature is invalid or the header format is incorrect
///
/// # Security
///
/// This function uses constant-time comparison to prevent timing attacks.
/// Always use this function to verify webhook requests before processing them.
///
/// # Example
///
/// ```
/// use web_app::webhook::whatsapp::security::verify_signature;
///
/// let header = "sha256=abc123def456...";
/// let payload = b"{\"object\":\"whatsapp_business_account\"}";
/// let secret = "your-app-secret";
///
/// if verify_signature(header, payload, secret) {
///     // Process the webhook
/// } else {
///     // Reject the request
/// }
/// ```
pub fn verify_signature(signature_header: &str, payload: &[u8], app_secret: &str) -> bool {
    // Extract the signature from the header (format: "sha256=<signature>")
    let signature_hex = match signature_header.strip_prefix("sha256=") {
        Some(sig) => sig,
        None => {
            logfire::warn!(
                "Invalid signature header format: expected 'sha256=' prefix"
            );
            return false;
        }
    };

    // Decode the hex signature
    let expected_signature = match hex::decode(signature_hex) {
        Ok(sig) => sig,
        Err(e) => {
            logfire::warn!(
                "Failed to decode signature hex: {error}",
                error = e.to_string()
            );
            return false;
        }
    };

    // Compute HMAC-SHA256 of the payload
    let mut mac = match HmacSha256::new_from_slice(app_secret.as_bytes()) {
        Ok(m) => m,
        Err(e) => {
            logfire::error!(
                "Failed to create HMAC instance: {error}",
                error = e.to_string()
            );
            return false;
        }
    };

    mac.update(payload);
    let computed_signature = mac.finalize().into_bytes();

    // Constant-time comparison to prevent timing attacks
    let is_valid: bool = computed_signature.ct_eq(&expected_signature[..]).into();

    if !is_valid {
        logfire::warn!("Webhook signature verification failed: signatures do not match");
    }

    is_valid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_signature_valid() {
        let payload = b"{\"test\":\"data\"}";
        let secret = "test_secret";

        // Generate a valid signature
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload);
        let signature = hex::encode(mac.finalize().into_bytes());
        let header = format!("sha256={}", signature);

        assert!(verify_signature(&header, payload, secret));
    }

    #[test]
    fn test_verify_signature_invalid() {
        let payload = b"{\"test\":\"data\"}";
        let secret = "test_secret";
        let wrong_signature = "sha256=0000000000000000000000000000000000000000000000000000000000000000";

        assert!(!verify_signature(wrong_signature, payload, secret));
    }

    #[test]
    fn test_verify_signature_wrong_secret() {
        let payload = b"{\"test\":\"data\"}";
        let secret = "test_secret";
        let wrong_secret = "wrong_secret";

        // Generate signature with wrong secret
        let mut mac = HmacSha256::new_from_slice(wrong_secret.as_bytes()).unwrap();
        mac.update(payload);
        let signature = hex::encode(mac.finalize().into_bytes());
        let header = format!("sha256={}", signature);

        assert!(!verify_signature(&header, payload, secret));
    }

    #[test]
    fn test_verify_signature_invalid_header_format() {
        let payload = b"{\"test\":\"data\"}";
        let secret = "test_secret";

        // Missing sha256= prefix
        assert!(!verify_signature("abc123", payload, secret));

        // Wrong prefix
        assert!(!verify_signature("sha1=abc123", payload, secret));
    }

    #[test]
    fn test_verify_signature_invalid_hex() {
        let payload = b"{\"test\":\"data\"}";
        let secret = "test_secret";

        // Invalid hex characters
        assert!(!verify_signature("sha256=zzzzz", payload, secret));
    }

    #[test]
    fn test_verify_signature_tampered_payload() {
        let original_payload = b"{\"test\":\"data\"}";
        let tampered_payload = b"{\"test\":\"hacked\"}";
        let secret = "test_secret";

        // Generate signature for original payload
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(original_payload);
        let signature = hex::encode(mac.finalize().into_bytes());
        let header = format!("sha256={}", signature);

        // Try to verify with tampered payload
        assert!(!verify_signature(&header, tampered_payload, secret));
    }
}
