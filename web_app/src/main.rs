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

use csrf::AesGcmCsrfProtection;
use logfire::config::MetricsOptions;
use ntex::web;
use ntex_cors::Cors;
use ntex_identity::{CookieIdentityPolicy, IdentityService};
use ntex_session::CookieSession;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use rust_decimal::prelude::ToPrimitive;

#[ntex::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging and metrics
    let shutdown_handler = logfire::configure()
        .install_panic_handler()
        .with_metrics(Some(MetricsOptions::default()))
        .send_to_logfire(logfire::config::SendToLogfire::Yes)
        .finish()?;

    // Initialize database connection pool
    let sqlite_repo = repo::sqlite::SqlxSqliteRepo {
        db_pool: utils::setup_sqlite_db_pool(config::APP_CONFIG.is_prod()).await?,
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
    let csrf_key =
        utils::build_csrf_key(&config::APP_CONFIG.csrf_pass, &config::APP_CONFIG.csrf_salt)?;
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

/// Configures SSL acceptor for production environments
fn setup_ssl_acceptor() -> anyhow::Result<openssl::ssl::SslAcceptorBuilder> {
    let mut ssl_acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls_server())
        .map_err(|e| anyhow::anyhow!("Failed to create SSL acceptor: {}", e))?;

    ssl_acceptor
        .set_private_key_file(&config::APP_CONFIG.private_key_path, SslFiletype::PEM)
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to load private key from {}: {}",
                config::APP_CONFIG.private_key_path,
                e
            )
        })?;

    ssl_acceptor
        .set_certificate_file(&config::APP_CONFIG.certificate_path, SslFiletype::PEM)
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to load certificate from {}: {}",
                config::APP_CONFIG.certificate_path,
                e
            )
        })?;

    Ok(ssl_acceptor)
}

/// Creates application state from the provided services
fn create_app_state(
    csrf_key: [u8; 32],
    sqlite_repo: repo::sqlite::SqlxSqliteRepo,
    storage_service: services::storage::StorageHandler,
    notification_service: services::notification::NotificationHandler,
) -> front::AppState {
    front::AppState {
        csrf_protec: AesGcmCsrfProtection::from_key(csrf_key),
        repo: Box::new(sqlite_repo),
        storage_service: Box::new(storage_service),
        notification_service: Box::new(notification_service),
    }
}

/// Configures and starts the web server with appropriate SSL settings
async fn configure_and_run_server(
    csrf_key: [u8; 32],
    session_key: [u8; 32],
    identity_key: [u8; 32],
    sqlite_repo: repo::sqlite::SqlxSqliteRepo,
    storage_service: services::storage::StorageHandler,
    notification_service: services::notification::NotificationHandler,
) -> anyhow::Result<()> {
    let server_addr = (
        "0.0.0.0",
        config::APP_CONFIG.wep_server_port.to_u16().unwrap_or(443),
    );

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
                    .secure(config::APP_CONFIG.is_prod())
                    .domain(config::APP_CONFIG.wep_server_host.to_string())
                    .max_age(consts::MAX_AGE_COOKIES)
                    .name("pet-info-session"),
            )
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&identity_key)
                    .name("user_id")
                    .domain(config::APP_CONFIG.wep_server_host.to_string())
                    .max_age(consts::MAX_AGE_COOKIES)
                    .secure(config::APP_CONFIG.is_prod()),
            ))
            .wrap(web::middleware::Logger::default())
            .wrap(web::middleware::Compress::default())
            .state(create_app_state(
                csrf_key,
                sqlite_repo.clone(),
                storage_service.clone(),
                notification_service.clone(),
            ))
            .configure(front::routes::pet_public_profile)
            .configure(front::routes::pet)
            .configure(front::routes::user_profile)
            .configure(front::routes::checkout)
            .configure(front::routes::blog)
            .configure(front::routes::reminders)
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

    let bound_server = if config::APP_CONFIG.is_prod() {
        let ssl_acceptor = setup_ssl_acceptor()?;
        server.bind_openssl(server_addr, ssl_acceptor)?
    } else {
        server.bind(server_addr)?
    };

    bound_server
        .run()
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))
}
