//! # Reminder API Module
//!
//! This module handles reminder and notification functionality, including
//! phone verification via WhatsApp, reminder scheduling, and notification
//! delivery for pet health and care reminders.

use crate::{config, metric, models, repo, services, utils};
use anyhow::bail;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use serde_json::json;

/// Sends a verification code to a phone number via WhatsApp.
///
/// Generates a TOTP verification code and sends it to the specified phone
/// number using WhatsApp Business API. Optimized with better error handling
/// and reduced nesting.
///
/// # Arguments
/// * `phone_number` - Phone number to send verification to (international format)
///
/// # Returns
/// * `anyhow::Result<()>` - Success confirmation or error details
pub async fn send_verification(phone_number: &str) -> anyhow::Result<()> {
    let otp = utils::TOTP_CLIENT.generate_current()?;

    let payload = create_whatsapp_verification_payload(phone_number, &otp);
    let response = send_whatsapp_message(payload).await?;

    if response.status().is_success() {
        return Ok(());
    }

    handle_whatsapp_error_response(response).await
}

/// Creates the WhatsApp verification message payload.
fn create_whatsapp_verification_payload(phone_number: &str, otp: &str) -> serde_json::Value {
    json!({
        "messaging_product": "whatsapp",
        "recipient_type": "individual",
        "to": phone_number,
        "type": "template",
        "template": {
            "name": "verify_reminder_phone_es",
            "language": {"code": "es"},
            "components": [
                {
                    "type": "body",
                    "parameters": [{"type": "text", "text": otp}]
                },
                {
                    "type": "button",
                    "sub_type": "url",
                    "index": "0",
                    "parameters": [{"type": "text", "text": otp}]
                }
            ]
        }
    })
}

/// Sends a message via WhatsApp Business API.
async fn send_whatsapp_message(payload: serde_json::Value) -> anyhow::Result<reqwest::Response> {
    utils::REQUEST_CLIENT
        .post(config::APP_CONFIG.whatsapp_send_msg_endpoint())
        .header("accept", "application/json")
        .header("content-type", "application/json")
        .bearer_auth(&config::APP_CONFIG.whatsapp_business_auth)
        .json(&payload)
        .send()
        .await
        .map_err(Into::into)
}

/// Handles WhatsApp API error responses.
async fn handle_whatsapp_error_response(response: reqwest::Response) -> anyhow::Result<()> {
    let error_body = response
        .json::<serde_json::Value>()
        .await
        .unwrap_or_else(|_| json!({"error": "Unknown error"}));

    logfire::error!("whatsapp_error={error}", error = error_body.to_string());

    bail!("WhatsApp API error: {}", error_body)
}

/// Validates a TOTP code against the current time window.
///
/// # Arguments
/// * `otp` - The OTP code to validate
///
/// # Returns
/// * `bool` - True if OTP is valid, false otherwise
pub fn validate_otp(otp: &str) -> bool {
    utils::TOTP_CLIENT.check_current(otp).unwrap_or(false)
}

/// Adds a verified phone number to a user's account.
pub async fn add_verified_phone_to_user(
    user_app_id: i64,
    phone: &str,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<()> {
    repo.insert_verified_phone_to_user_app(user_app_id, phone)
        .await
}

/// Removes the verified phone number from a user's account.
pub async fn remove_verified_phone_to_user(
    user_app_id: i64,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<()> {
    repo.set_to_null_verified_phone(user_app_id).await
}

/// Information required to schedule a reminder notification.
///
/// Contains all the details needed to create and schedule a reminder,
/// including user information, timing, and message content.
#[derive(Debug)]
pub struct ScheduleReminderInfo {
    /// ID of the user to send the reminder to
    pub user_id: i64,
    /// Phone number to send the reminder to
    pub phone_number: String,
    /// When to send the reminder (with timezone)
    pub when: DateTime<Tz>,
    /// Message content for the reminder
    pub body: String,
}

/// Schedules a reminder notification for future delivery.
pub async fn schedule_reminder(
    reminder_info: ScheduleReminderInfo,
    repo: &repo::ImplAppRepo,
    notification_service: &services::ImplNotificationService,
) -> anyhow::Result<()> {
    let execution_id = notification_service
        .send_reminder_to_phone_number(&reminder_info)
        .await?;

    repo.insert_user_remider(&create_reminder_model(reminder_info, execution_id))
        .await?;
    metric::incr_reminder_action_statds("schedule");

    Ok(())
}

/// Creates a reminder model from the provided information.
fn create_reminder_model(
    reminder_info: ScheduleReminderInfo,
    execution_id: String,
) -> models::reminder::Reminder {
    models::reminder::Reminder {
        id: 0,
        user_app_id: reminder_info.user_id,
        body: reminder_info.body,
        execution_id,
        notification_type: models::reminder::ReminderNotificationType::WhatsApp,
        user_timezone: reminder_info.when.timezone().name().to_string(),
        send_at: reminder_info.when.to_utc(),
        created_at: Utc::now(),
    }
}

/// Retrieves all scheduled reminders for a user.
///
/// Fetches all active (non-sent, non-cancelled) reminders that are
/// scheduled for the specified user.
///
/// # Arguments
/// * `user_app_id` - ID of the user to get reminders for
/// * `repo` - Repository instance for database operations
///
/// # Returns
/// * `anyhow::Result<Vec<models::reminder::Reminder>>` - List of scheduled reminders
pub async fn get_scheduled_reminders(
    user_app_id: i64,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<Vec<models::reminder::Reminder>> {
    repo.get_active_user_remiders(user_app_id).await
}

/// Deletes a scheduled reminder and cancels its delivery.
///
/// Removes a reminder from the database and cancels its scheduled
/// delivery through the notification service to prevent it from
/// being sent.
///
/// # Arguments
/// * `reminder_id` - ID of the reminder to delete
/// * `user_id` - ID of the user who owns the reminder
/// * `repo` - Repository instance for database operations
/// * `notification_service` - Service for cancelling scheduled notifications
///
/// # Returns
/// * `anyhow::Result<()>` - Success confirmation or error details
///
/// # Process
/// 1. Retrieve execution ID for the reminder
/// 2. Cancel scheduled notification if execution ID exists
/// 3. Delete reminder from database
/// 4. Record cancellation metrics
///
/// # Errors
/// Returns an error if:
/// - Database operations fail
/// - Notification service cancellation fails
/// - User doesn't own the specified reminder
pub async fn delete_reminder(
    reminder_id: i64,
    user_id: i64,
    repo: &repo::ImplAppRepo,
    notification_service: &services::ImplNotificationService,
) -> anyhow::Result<()> {
    if let Some(execution_id) = repo.get_reminder_execution_id(user_id, reminder_id).await? {
        notification_service
            .cancel_reminder_to_phone_number(&execution_id)
            .await?;

        metric::incr_reminder_action_statds("cancel");
    }

    repo.delete_user_reminder(reminder_id, user_id).await
}
