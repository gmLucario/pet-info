//! Internal API endpoints for Lambda function callbacks.
//!
//! These endpoints are not exposed to the public internet and are only
//! called by internal Lambda functions. They require authentication via
//! the X-Internal-Secret header.

use crate::{api, config, front::AppState};
use chrono::{DateTime, Utc};
use ntex::web;
use serde::Deserialize;

/// Verifies the internal API secret from the request headers.
fn verify_internal_secret(req: &web::HttpRequest) -> bool {
    let secret = req
        .headers()
        .get("X-Internal-Secret")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if let Ok(Some(config)) = config::APP_CONFIG.get() {
        !config.internal_api_secret.is_empty() && secret == config.internal_api_secret
    } else {
        false
    }
}

#[derive(Deserialize)]
pub struct RescheduleRequest {
    pub reminder_id: i64,
    pub new_execution_id: String,
    pub new_send_at: DateTime<Utc>,
}

/// Updates a reminder's execution details after the Lambda reschedules it.
///
/// Called by the send-reminders Lambda after scheduling the next occurrence
/// of a recurring reminder.
#[web::post("/reschedule")]
pub async fn reschedule_reminder(
    req: web::HttpRequest,
    form: web::types::Json<RescheduleRequest>,
    app_state: web::types::State<AppState>,
) -> impl web::Responder {
    if !verify_internal_secret(&req) {
        return web::HttpResponse::Unauthorized().finish();
    }

    match api::reminder::reschedule_reminder(
        form.reminder_id,
        form.new_execution_id.clone(),
        form.new_send_at,
        &app_state.repo,
    )
    .await
    {
        Ok(_) => web::HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::error!("Failed to reschedule reminder: {}", e);
            web::HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Deserialize)]
pub struct CheckReminderPath {
    pub reminder_id: i64,
}

#[derive(serde::Serialize)]
pub struct ReminderActiveResponse {
    pub active: bool,
}

/// Checks if a reminder is still active (exists and not deleted).
///
/// Called by the send-reminders Lambda before sending a message to ensure
/// the reminder hasn't been deleted by the user.
#[web::get("/reminder/{reminder_id}/active")]
pub async fn check_reminder_active(
    req: web::HttpRequest,
    path: web::types::Path<CheckReminderPath>,
    app_state: web::types::State<AppState>,
) -> impl web::Responder {
    if !verify_internal_secret(&req) {
        return web::HttpResponse::Unauthorized().finish();
    }

    // Check if reminder exists by trying to get its execution_id
    // We use a query that doesn't require user_id since Lambda doesn't have it
    match app_state
        .repo
        .check_reminder_exists(path.reminder_id)
        .await
    {
        Ok(exists) => web::HttpResponse::Ok().json(&ReminderActiveResponse { active: exists }),
        Err(e) => {
            tracing::error!("Failed to check reminder: {}", e);
            web::HttpResponse::InternalServerError().finish()
        }
    }
}
