use crate::config;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
};
use std::str::FromStr;
use std::sync::LazyLock;
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

pub static REQUEST_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

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
