use crate::config;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
};
use std::str::FromStr;

pub async fn run_migrations(db_pool: &sqlx::SqlitePool, file_name: &str) -> anyhow::Result<()> {
    let mut tera = tera::Tera::new("../migrations/**/*.sql")?;
    tera.autoescape_on(vec![".sql"]);

    let create_tables_query = tera.render(file_name, &tera::Context::new())?;

    sqlx::query(&create_tables_query).execute(db_pool).await?;
    Ok(())
}

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
