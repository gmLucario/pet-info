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

use anyhow::anyhow;
use argon2::Argon2;
use csrf::AesGcmCsrfProtection;
use ntex::web;
use ntex_cors::Cors;
use ntex_identity::{CookieIdentityPolicy, IdentityService};
use ntex_session::CookieSession;
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
            Uuid::new_v4().as_bytes(),
            Uuid::new_v4().as_bytes() as &[u8],
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
            .wrap(Cors::default())
            .wrap(CookieSession::private(&[0; 32]).secure(config::APP_CONFIG.is_prod()))
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32])
                    .name("user_id")
                    .path("/")
                    .domain(config::APP_CONFIG.wep_server_host.to_string())
                    .max_age(chrono::TimeDelta::days(1).num_seconds())
                    .secure(config::APP_CONFIG.is_prod()),
            ))
            .wrap(front::middleware::csrf_token::CsrfToken)
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
                front::server::serve_dog_hi,
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
    Ok(server.bind(("0.0.0.0", 80))?.run().await?)
}
