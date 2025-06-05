#![recursion_limit = "256"]

pub mod api;
pub mod config;
pub mod consts;
pub mod front;
pub mod logger;
pub mod models;
pub mod repo;
pub mod services;
pub mod utils;

use std::str::FromStr;

use anyhow::anyhow;
use argon2::Argon2;
use csrf::AesGcmCsrfProtection;
use ntex::web;
use ntex_cors::Cors;
use ntex_identity::{CookieIdentityPolicy, IdentityService};
use ntex_session::CookieSession;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use rust_decimal::prelude::ToPrimitive;
use uuid::Uuid;

#[ntex::main]
async fn main() -> anyhow::Result<()> {
    logger::setup_simple_logger()?;

    let db_pool = utils::setup_sqlite_db_pool(config::APP_CONFIG.is_prod()).await?;

    let sqlite_repo = repo::sqlite::SqlxSqliteRepo {
        db_pool: db_pool.clone(),
    };

    let mut csrf_key = [0u8; 32];
    Argon2::default()
        .hash_password_into(
            Uuid::from_str(&config::APP_CONFIG.csrf_pass)?.as_bytes(),
            Uuid::from_str(&config::APP_CONFIG.csrf_salt)?.as_bytes(),
            &mut csrf_key,
        )
        .map_err(|err| anyhow!("csrf_key couldn't be created: {}", err))?;

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
                CookieSession::private(&[0; 64])
                    .secure(config::APP_CONFIG.is_prod())
                    .domain(config::APP_CONFIG.wep_server_host.to_string())
                    .max_age(consts::MAX_AGE_COOKIES)
                    .name("pet-info-session"),
            )
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 64])
                    .name("user_id")
                    .domain(config::APP_CONFIG.wep_server_host.to_string())
                    .max_age(consts::MAX_AGE_COOKIES)
                    .secure(config::APP_CONFIG.is_prod()),
            ))
            .wrap(web::middleware::Logger::default())
            .wrap(web::middleware::Compress::default())
            .state(front::AppState {
                csrf_protec: AesGcmCsrfProtection::from_key(csrf_key),
                repo: Box::new(sqlite_repo.clone()),
                storage_service: Box::new(storage_service.clone()),
                notification_service: Box::new(notification_service.clone()),
            })
            .configure(front::routes::pet)
            .configure(front::routes::profile)
            .configure(front::routes::checkout)
            .configure(front::routes::blog)
            .configure(front::routes::reminders)
            .service((
                ntex_files::Files::new("/static", "web/static/"),
                front::server::serve_favicon,
                front::server::index,
                front::server::google_callback,
                front::server::get_reactivate_account_view,
                front::server::reactivate_account,
            ))
            .default_service(
                web::route()
                    .guard(web::guard::Not(web::guard::Get()))
                    .to(front::server::serve_not_found),
            )
    });

    let server_addr = (
        "0.0.0.0",
        config::APP_CONFIG.wep_server_port.to_u16().unwrap_or(443),
    );
    let server = if config::APP_CONFIG.is_prod() {
        let mut ssl_server = SslAcceptor::mozilla_intermediate(SslMethod::tls_server())?;
        ssl_server.set_private_key_file(&config::APP_CONFIG.private_key_path, SslFiletype::PEM)?;
        ssl_server.set_certificate_file(&config::APP_CONFIG.certificate_path, SslFiletype::PEM)?;

        server.bind_openssl(server_addr, ssl_server)
    } else {
        server.bind(server_addr)
    };

    Ok(server?.run().await?)
}
