use crate::config;
use lazy_static::lazy_static;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    SqlitePool,
};
use std::str::FromStr;
use totp_rs::{Algorithm, Secret, TOTP};

pub async fn setup_sqlite_db_pool(encrypted: bool) -> anyhow::Result<SqlitePool> {
    if encrypted {
        return Ok(SqlitePool::connect_with(
            SqliteConnectOptions::from_str(&config::APP_CONFIG.db_host)?
                .pragma("key", &config::APP_CONFIG.db_pass_encrypt)
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
        SqliteConnectOptions::from_str(&config::APP_CONFIG.db_host)?.pragma("foreign_keys", "ON"),
    )
    .await?)
}

lazy_static! {
    pub static ref request_client: reqwest::Client = reqwest::Client::new();
}

lazy_static! {
    pub static ref totp_client: TOTP = TOTP::new(
        Algorithm::SHA512,
        6,
        1,
        60,
        Secret::Raw(config::OTP_SECRET.as_bytes().to_vec())
            .to_bytes()
            .unwrap(),
    )
    .unwrap();
}
