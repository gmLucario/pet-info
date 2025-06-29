//! Helper functions could be used in api/, front/, ...

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

pub fn build_csrf_key(pwd: &str, salt: &str) -> anyhow::Result<[u8; 32]> {
    let mut csrf_key = [0u8; 32];
    Argon2::default()
        .hash_password_into(
            Uuid::from_str(pwd)?.as_bytes(),
            Uuid::from_str(salt)?.as_bytes(),
            &mut csrf_key,
        )
        .map_err(|err| anyhow!("csrf_key couldn't be created: {}", err))?;

    Ok(csrf_key)
}

pub fn build_random_csrf_key() -> anyhow::Result<[u8; 32]> {
    build_csrf_key(&Uuid::new_v4().to_string(), &Uuid::new_v4().to_string())
}

/// Client to make http requests
pub static REQUEST_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

/// Time-based one time password  client
pub static TOTP_CLIENT: LazyLock<TOTP> = LazyLock::new(|| {
    TOTP::new(
        Algorithm::SHA512,
        6,
        1,
        60,
        Secret::Raw(config::OTP_SECRET.as_bytes().to_vec())
            .to_bytes()
            .unwrap(),
    )
    .unwrap()
});
