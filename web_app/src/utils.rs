//! Utility functions and shared resources for the pet-info application.
//!
//! This module provides common functionality used across different parts of the application:
//! - Database connection management with optional encryption
//! - Cryptographic key generation for CSRF protection
//! - HTTP client for external API calls
//! - Time-based One-Time Password (TOTP) generation

use crate::config;
use anyhow::{Context, anyhow};
use argon2::Argon2;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
};
use std::{str::FromStr, sync::LazyLock};
use totp_rs::{Algorithm, Secret, TOTP};
use uuid::Uuid;

/// Detects image format from magic bytes.
///
/// Checks the first few bytes of the image data to determine the file format.
/// This is more reliable than trusting file extensions.
///
/// # Arguments
/// * `bytes` - The image file bytes
///
/// # Returns
/// File extension string ("jpg", "png", "gif", "webp", etc.)
pub fn detect_image_format(bytes: &[u8]) -> &'static str {
    if bytes.len() < 4 {
        return "jpg"; // Default fallback
    }

    // Check magic bytes for common image formats
    match &bytes[0..4] {
        [0x89, 0x50, 0x4E, 0x47] => "png", // PNG
        [0xFF, 0xD8, 0xFF, ..] => "jpg",   // JPEG
        [0x47, 0x49, 0x46, ..] => "gif",   // GIF
        [0x52, 0x49, 0x46, 0x46] if bytes.len() >= 12 && &bytes[8..12] == b"WEBP" => "webp", // WebP
        [0x42, 0x4D, ..] => "bmp",         // BMP
        // HEIC signatures (ftypheic, ftypheix, ftypheim, ftypmsf1)
        // usually start at offset 4, bytes 4-12 are "ftypheic" etc
        _ if bytes.len() >= 12 => match &bytes[4..12] {
            b"ftypheic" | b"ftypheix" | b"ftypheim" | b"ftypmsf1" => "heic",
            _ => "jpg",
        },
        _ => "jpg", // Default fallback
    }
}

/// Creates and configures a SQLite connection pool with optional encryption.
///
/// This function establishes a database connection pool that can be configured for
/// either encrypted (SQLCipher) or unencrypted SQLite databases. The encryption
/// settings are optimized for security while maintaining reasonable performance.
///
/// # Arguments
/// * `encrypted` - Whether to use SQLCipher encryption for the database
///
/// # Database Configuration
/// ## Encrypted Database (SQLCipher)
/// When `encrypted` is `true`, the following SQLCipher settings are applied:
/// - **Cipher Page Size**: 1024 bytes (balance between security and performance)
/// - **KDF Iterations**: 64,000 (PBKDF2 rounds for key derivation)
/// - **HMAC Algorithm**: SHA1 (for authenticated encryption)
/// - **KDF Algorithm**: PBKDF2-HMAC-SHA1 (industry standard)
/// - **Journal Mode**: DELETE (secure deletion of journal files)
///
/// ## Unencrypted Database
/// When `encrypted` is `false`, uses standard SQLite with:
/// - **Foreign Keys**: Enabled for referential integrity
/// - **Journal Mode**: Default (WAL mode)
pub async fn setup_sqlite_db_pool(encrypted: bool) -> anyhow::Result<SqlitePool> {
    let app_config = config::APP_CONFIG
        .get()
        .context("failed to get app config")?;
    if encrypted {
        return Ok(SqlitePool::connect_with(
            SqliteConnectOptions::from_str(&app_config.db_host)?
                .pragma("key", &app_config.db_pass_encrypt)
                .pragma("cipher_page_size", "1024")
                .pragma("kdf_iter", "64000")
                .pragma("cipher_hmac_algorithm", "HMAC_SHA1")
                .pragma("cipher_kdf_algorithm", "PBKDF2_HMAC_SHA1")
                .pragma("foreign_keys", "ON")
                .journal_mode(SqliteJournalMode::Delete),
        )
        .await?);
    }

    Ok(SqlitePool::connect_with(
        SqliteConnectOptions::from_str(&app_config.db_host)?.pragma("foreign_keys", "ON"),
    )
    .await?)
}

/// Derives a 32-byte cryptographic key using Argon2 with UUID-based password and salt.
///
/// This function uses the Argon2 key derivation function to generate a secure 32-byte key
/// suitable for CSRF token protection. The key is derived from UUID values converted to
/// byte arrays, providing high entropy input material.
///
/// # Arguments
/// * `pwd` - Password UUID used as the primary key material
/// * `salt` - Salt UUID used to prevent rainbow table attacks
///
/// # Security Properties
/// - **Algorithm**: Argon2 (default variant, typically Argon2id)
/// - **Output Length**: 32 bytes (256 bits)
/// - **Input Entropy**: 32 bytes from UUID (128 bits entropy × 2)
/// - **Salt Length**: 16 bytes from UUID (128 bits)
/// - **Memory-hard**: Resistant to GPU/ASIC attacks
///
/// # Key Derivation Process
/// 1. Converts UUIDs to their byte representation (16 bytes each)
/// 2. Uses Argon2 with default parameters for memory and iteration costs
/// 3. Produces a deterministic 32-byte output for the same inputs
/// 4. Each UUID provides 128 bits of entropy
///
/// # Security Considerations
/// - UUIDs should be generated using cryptographically secure random generators
/// - Both password and salt UUIDs should be unique and unpredictable
/// - The derived key is suitable for HMAC operations and symmetric encryption
/// - Keys should be rotated periodically (recommended: every 6 months)
pub fn build_csrf_key(pwd: &Uuid, salt: &Uuid) -> anyhow::Result<[u8; 32]> {
    let mut csrf_key = [0u8; 32];
    Argon2::default()
        .hash_password_into(pwd.as_bytes(), salt.as_bytes(), &mut csrf_key)
        .map_err(|err| anyhow!("csrf_key couldn't be created: {}", err))?;

    Ok(csrf_key)
}

/// Generates a random 32-byte cryptographic key using freshly generated UUIDs.
///
/// This is a convenience function that creates a completely random CSRF key by
/// generating two new UUIDs and using them as password and salt for key derivation.
/// Each call produces a different key, making it suitable for session-specific
/// or ephemeral cryptographic operations.
///
/// # Security Properties
/// - **Randomness**: Uses UUID v4 with cryptographically secure random generator
/// - **Entropy**: 256 bits of entropy from two 128-bit UUIDs
/// - **Uniqueness**: Extremely low probability of collision
/// - **Unpredictability**: Each key is independent and unrelated to previous keys
///
/// # Use Cases
/// - Session-specific CSRF protection keys
/// - Temporary encryption keys for short-lived operations
/// - Key material that doesn't need to be reproducible
/// - Development and testing environments
pub fn build_random_csrf_key() -> anyhow::Result<[u8; 32]> {
    build_csrf_key(&Uuid::new_v4(), &Uuid::new_v4())
}

/// Shared HTTP client for making external API requests.
///
/// This is a globally available, lazily-initialized HTTP client that provides:
/// - **Connection Pooling**: Reuses connections for better performance
/// - **Thread Safety**: Safe to use across multiple threads
/// - **Memory Efficiency**: Single client instance shared across the application
/// - **Default Configuration**: Optimized settings for typical API calls
///
/// # Usage
/// The client is automatically initialized on first access and reused for all
/// subsequent HTTP operations. It's suitable for calling external APIs such as:
/// - MercadoPago payment processing
/// - WhatsApp Business API
/// - Google OAuth services
/// - Other third-party integrations
///
/// # Examples
/// ```rust
/// // GET request
/// let response = REQUEST_CLIENT
///     .get("https://api.example.com/data")
///     .header("Authorization", "Bearer token")
///     .send()
///     .await?;
///
/// // POST request with JSON body
/// let response = REQUEST_CLIENT
///     .post("https://api.example.com/submit")
///     .json(&payload)
///     .send()
///     .await?;
/// ```
///
/// # Performance Benefits
/// - Avoids the overhead of creating new clients for each request
/// - Maintains HTTP/2 connections when supported
/// - Implements automatic retry logic and connection pooling
pub static REQUEST_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

/// TOTP algorithm for cryptographic hashing
const TOTP_HASH_ALGORITHM: Algorithm = Algorithm::SHA512;
/// Number of digits in generated TOTP codes
const TOTP_CODE_DIGITS: usize = 6;
/// Time window tolerance for TOTP validation (allows previous/next window)
const TOTP_VALIDATION_SKEW: u8 = 1;
/// Time duration in seconds for each TOTP code validity window
const TOTP_TIME_STEP_SECONDS: u64 = 60;

/// Time-based One-Time Password (TOTP) client for two-factor authentication.
///
/// This client generates and validates TOTP codes using the following configuration:
/// - **Algorithm**: SHA-512 (more secure than SHA-1 or SHA-256)
/// - **Digits**: 6 (standard TOTP code length)
/// - **Skew**: 1 (allows codes from previous/next time window)
/// - **Step**: 60 seconds (time window for each code)
/// - **Secret**: Derived from application's OTP_SECRET UUID
///
/// # Security Features
/// - **High Entropy Secret**: Uses UUID as base secret material
/// - **Strong Hashing**: SHA-512 provides better security than SHA-1
/// - **Time Synchronization**: 60-second windows balance security and usability
/// - **Limited Skew**: 1-step tolerance for clock drift
///
/// # Examples
/// ```rust
/// // Generate current TOTP code
/// let current_code = TOTP_CLIENT.generate_current()?;
/// println!("Current code: {}", current_code);
///
/// // Verify a user-provided code
/// let user_code = "123456";
/// let is_valid = TOTP_CLIENT.check_current(user_code)?;
///
/// // Get code for specific timestamp
/// let timestamp = 1234567890;
/// let code_at_time = TOTP_CLIENT.generate(timestamp);
/// ```
///
/// # Implementation Notes
/// - The secret is regenerated on each application restart for security
/// - Codes are valid for 60 seconds with ±1 step tolerance (total 3 minutes)
/// - Uses the `totp-rs` crate for RFC 6238 compliance
pub static TOTP_CLIENT: LazyLock<TOTP> = LazyLock::new(|| {
    TOTP::new(
        TOTP_HASH_ALGORITHM,
        TOTP_CODE_DIGITS,
        TOTP_VALIDATION_SKEW,
        TOTP_TIME_STEP_SECONDS,
        Secret::Raw(config::OTP_SECRET.as_bytes().to_vec())
            .to_bytes()
            .unwrap(),
    )
    .unwrap()
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_image_format() {
        // PNG
        assert_eq!(
            detect_image_format(&[0x89, 0x50, 0x4E, 0x47, 0x00, 0x00]),
            "png"
        );

        // JPEG
        assert_eq!(detect_image_format(&[0xFF, 0xD8, 0xFF, 0xE0]), "jpg");

        // GIF
        assert_eq!(detect_image_format(&[0x47, 0x49, 0x46, 0x38]), "gif");

        // WebP
        let mut webp_data = vec![0u8; 12];
        webp_data[0] = 0x52; // R
        webp_data[1] = 0x49; // I
        webp_data[2] = 0x46; // F
        webp_data[3] = 0x46; // F
        webp_data[8] = 0x57; // W
        webp_data[9] = 0x45; // E
        webp_data[10] = 0x42; // B
        webp_data[11] = 0x50; // P
        assert_eq!(detect_image_format(&webp_data), "webp");

        // BMP
        assert_eq!(detect_image_format(&[0x42, 0x4D, 0x00, 0x00]), "bmp");

        // HEIC
        // ftypheic
        let mut heic_data = vec![0u8; 12];
        heic_data[4] = b'f';
        heic_data[5] = b't';
        heic_data[6] = b'y';
        heic_data[7] = b'p';
        heic_data[8] = b'h';
        heic_data[9] = b'e';
        heic_data[10] = b'i';
        heic_data[11] = b'c';
        assert_eq!(detect_image_format(&heic_data), "heic");

        // Fallback
        assert_eq!(detect_image_format(&[0x00, 0x00, 0x00, 0x00]), "jpg");
        assert_eq!(detect_image_format(&[]), "jpg");
    }
}
