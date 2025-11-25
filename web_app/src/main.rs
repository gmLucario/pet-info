//! # Pet Info Web Application
//!
//! Main entry point for the pet information management web application.
//! Configures SSL, middleware, cryptographic keys, and route handling.

#![recursion_limit = "256"]

pub mod api;
pub mod config;
pub mod consts;
pub mod front;
pub mod logger;
pub mod metric;
pub mod models;
pub mod repo;
pub mod services;
pub mod utils;
pub mod webhook;

use anyhow::Context;
use csrf::AesGcmCsrfProtection;
use logfire::config::MetricsOptions;
use ntex::web;
use ntex_cors::Cors;
use ntex_identity::{CookieIdentityPolicy, IdentityService};
use ntex_session::CookieSession;

#[ntex::main]
async fn main() -> anyhow::Result<()> {
    // Initialize configuration
    config::init_config().await?;

    let app_config = config::APP_CONFIG
        .get()
        .context("failed to get app config")?;

    // Initialize logging and metrics
    let shutdown_handler = logfire::configure()
        .install_panic_handler()
        .with_metrics(Some(MetricsOptions::default()))
        .send_to_logfire(logfire::config::SendToLogfire::Yes)
        .with_token(&app_config.logfire_token)
        .finish()?;

    // Initialize database connection pool
    let sqlite_repo = repo::sqlite::SqlxSqliteRepo {
        db_pool: utils::setup_sqlite_db_pool(app_config.is_prod()).await?,
    };

    // Initialize AWS services
    let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(aws_config::Region::new("us-east-2"))
        .load()
        .await;

    let storage_service = services::storage::StorageHandler {
        client: aws_sdk_s3::Client::new(&aws_config),
    };
    let notification_service = services::notification::NotificationHandler {
        client: aws_sdk_sfn::Client::new(&aws_config),
    };

    // Generate cryptographically secure keys for application security
    // All keys are derived from configured password and salt using Argon2
    let csrf_key = utils::build_csrf_key(&app_config.csrf_pass, &app_config.csrf_salt)?;
    let session_key = utils::build_random_csrf_key()?;
    let identity_key = utils::build_random_csrf_key()?;

    // Configure and start the web server
    configure_and_run_server(
        csrf_key,
        session_key,
        identity_key,
        sqlite_repo,
        storage_service,
        notification_service,
    )
    .await?;

    shutdown_handler.shutdown()?;

    Ok(())
}

/// Creates application state from the provided services
fn create_app_state(
    csrf_key: [u8; 32],
    sqlite_repo: repo::sqlite::SqlxSqliteRepo,
    storage_service: services::storage::StorageHandler,
    notification_service: services::notification::NotificationHandler,
) -> anyhow::Result<front::AppState> {
    let whatsapp_client = webhook::whatsapp::client::WhatsAppClient::new()?;

    Ok(front::AppState {
        csrf_protec: AesGcmCsrfProtection::from_key(csrf_key),
        repo: Box::new(sqlite_repo),
        storage_service: Box::new(storage_service),
        notification_service: Box::new(notification_service),
        whatsapp_client,
    })
}

/// Configures and starts the web server
/// Note: Server runs on localhost:8080 (HTTP only)
/// TLS/mTLS is handled by Nginx reverse proxy
async fn configure_and_run_server(
    csrf_key: [u8; 32],
    session_key: [u8; 32],
    identity_key: [u8; 32],
    sqlite_repo: repo::sqlite::SqlxSqliteRepo,
    storage_service: services::storage::StorageHandler,
    notification_service: services::notification::NotificationHandler,
) -> anyhow::Result<()> {
    let app_config = config::APP_CONFIG
        .get()
        .context("failed to get app config")?;

    // Server binds to localhost:8080
    // Nginx reverse proxy handles HTTPS/TLS/mTLS on port 443
    let server_addr = ("127.0.0.1", 8080);

    let server = web::server(move || {
        web::App::new()
            .wrap(
                Cors::new()
                    .allowed_methods(vec![
                        "GET", "HEAD", "POST", "OPTIONS", "PUT", "PATCH", "DELETE",
                    ])
                    .allowed_origin("http://localhost:8080")
                    .allowed_origin("https://openidconnect.googleapis.com")
                    .allowed_origin("https://pet-info.link")
                    .allowed_origin("https://oauth2.googleapis.com")
                    .allowed_origin("https://www.googleapis.com")
                    .allowed_origin("https://accounts.google.com")
                    .allowed_origin("https://graph.facebook.com")
                    .allowed_origin("https://api.mercadopago.com")
                    .finish(),
            )
            .wrap(
                CookieSession::private(&session_key)
                    .secure(app_config.is_prod())
                    .max_age(consts::MAX_AGE_COOKIES)
                    .name("pet-info-session"),
            )
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&identity_key)
                    .name("user_id")
                    .max_age(consts::MAX_AGE_COOKIES)
                    .secure(app_config.is_prod()),
            ))
            .wrap(web::middleware::Logger::default())
            .wrap(web::middleware::Compress::default())
            .state(
                create_app_state(
                    csrf_key,
                    sqlite_repo.clone(),
                    storage_service.clone(),
                    notification_service.clone(),
                )
                .expect("Failed to create app state"),
            )
            .configure(front::routes::pet_public_profile)
            .configure(front::routes::pet)
            .configure(front::routes::user_profile)
            .configure(front::routes::checkout)
            .configure(front::routes::blog)
            .configure(front::routes::reminders)
            .configure(front::routes::webhooks)
            .service((
                ntex_files::Files::new("/static", "web/static/"),
                front::server::serve_favicon,
                front::server::index,
                front::auth::google_callback,
                front::server::get_reactivate_account_view,
                front::server::reactivate_account,
            ))
            .default_service(
                web::route()
                    .guard(web::guard::Not(web::guard::Get()))
                    .to(front::server::serve_not_found),
            )
    });

    server
        .bind(server_addr)?
        .run()
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))
}
