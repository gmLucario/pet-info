use chrono::{DateTime, Months, Utc};
use lambda_runtime::{tracing, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config;

/// Full payload from Step Function
#[derive(Deserialize, Debug)]
pub struct IncomingMessage {
    /// Original scheduled time
    pub when: String,
    /// Reminder data (phone and body)
    pub reminder: ReminderData,
    /// Optional repeat configuration for recurring reminders
    #[serde(default)]
    pub repeat_config: Option<RepeatConfig>,
    /// Reminder ID in database (needed for rescheduling)
    #[serde(default)]
    pub reminder_id: Option<i64>,
    /// User's timezone (needed for rescheduling)
    #[serde(default)]
    pub user_timezone: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ReminderData {
    pub phone: String,
    pub body: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RepeatConfig {
    pub repeat_type: String,
    pub repeat_interval: i32,
}

#[derive(Serialize)]
pub struct OutgoingMessage {
    pub req_id: String,
    pub msg: String,
}

/// Check if reminder still exists (not deleted by user)
async fn check_reminder_active(reminder_id: i64) -> Result<bool, Error> {
    let url = format!(
        "{}/api/internal/reminder/{}/active",
        config::APP_CONFIG.web_app_api_url, reminder_id
    );

    let response = reqwest::Client::new()
        .get(&url)
        .header("X-Internal-Secret", &config::APP_CONFIG.internal_api_secret)
        .send()
        .await?;

    if !response.status().is_success() {
        tracing::warn!("Failed to check reminder status: {}", response.status());
        // If we can't check, assume it's active to avoid missing reminders
        return Ok(true);
    }

    #[derive(Deserialize)]
    struct ActiveResponse {
        active: bool,
    }

    let result: ActiveResponse = response.json().await?;
    Ok(result.active)
}

async fn send_msg(phone_number: &str, body: &str) -> Result<(), Error> {
    let response = reqwest::Client::new()
        .post(config::APP_CONFIG.whatsapp_send_msg_endpoint())
        .header("accept", "application/json")
        .header("content-type", "application/json")
        .bearer_auth(config::APP_CONFIG.whatsapp_business_auth.to_string())
        .json(&json!({
        "messaging_product": "whatsapp",
        "recipient_type": "individual",
        "to": phone_number,
        "type": "template",
        "template": {
            "name": "pet_reminder_mex",
            "language": {"code": "es_mx"},
            "components": [{
                "type": "body",
                "parameters": [{
                    "type": "text",
                    "parameter_name": "reminder_txt",
                    "text": body,
            }]}]
        }}))
        .send()
        .await?;

    if response.status().is_success() {
        return Ok(());
    }

    let response = response
        .json::<serde_json::Value>()
        .await
        .unwrap_or_default();

    Err(Box::new(simple_error::SimpleError::new(format!(
        "reminder did not send: {response}"
    ))))
}

fn calculate_next_execution(repeat_type: &str, interval: i32) -> DateTime<Utc> {
    let now = Utc::now();
    match repeat_type {
        "daily" => now + chrono::Duration::days(interval as i64),
        "weekly" => now + chrono::Duration::weeks(interval as i64),
        "monthly" => now
            .checked_add_months(Months::new(interval as u32))
            .unwrap_or(now),
        "yearly" => now
            .checked_add_months(Months::new((interval * 12) as u32))
            .unwrap_or(now),
        _ => now + chrono::Duration::days(1), // fallback to daily
    }
}

/// Generates a unique execution name for a reminder.
/// Format: `reminder-{reminder_id}-{timestamp}`
/// IMPORTANT: Must match the format used in web_app/src/services/notification.rs
fn generate_execution_name(reminder_id: i64) -> String {
    format!("reminder-{}-{}", reminder_id, Utc::now().timestamp_millis())
}

async fn schedule_next_reminder(
    payload: &IncomingMessage,
    repeat_config: &RepeatConfig,
    reminder_id: i64,
    timezone: &str,
) -> Result<String, Error> {
    let next_when = calculate_next_execution(&repeat_config.repeat_type, repeat_config.repeat_interval);

    tracing::info!(
        "Scheduling next reminder for {} at {}",
        reminder_id,
        next_when.to_rfc3339()
    );

    // Build new payload for next execution
    let new_payload = json!({
        "when": next_when.to_rfc3339(),
        "reminder": {
            "phone": payload.reminder.phone,
            "body": payload.reminder.body
        },
        "repeat_config": {
            "repeat_type": repeat_config.repeat_type,
            "repeat_interval": repeat_config.repeat_interval
        },
        "reminder_id": reminder_id,
        "user_timezone": timezone
    });

    // Start new Step Function execution with named execution for tracking
    let config = aws_config::load_from_env().await;
    let sfn_client = aws_sdk_sfn::Client::new(&config);

    let result = sfn_client
        .start_execution()
        .state_machine_arn(&config::APP_CONFIG.step_function_arn)
        .name(generate_execution_name(reminder_id))
        .input(new_payload.to_string())
        .send()
        .await?;

    let new_execution_arn = result.execution_arn().to_string();

    tracing::info!("Started new Step Function execution: {}", new_execution_arn);

    // Update reminder in database with new execution details
    update_reminder_in_db(reminder_id, &new_execution_arn, next_when).await?;

    Ok(new_execution_arn)
}

async fn update_reminder_in_db(
    reminder_id: i64,
    new_execution_id: &str,
    new_send_at: DateTime<Utc>,
) -> Result<(), Error> {
    let url = format!(
        "{}/api/internal/reschedule",
        config::APP_CONFIG.web_app_api_url
    );

    let response = reqwest::Client::new()
        .post(&url)
        .header("X-Internal-Secret", &config::APP_CONFIG.internal_api_secret)
        .json(&json!({
            "reminder_id": reminder_id,
            "new_execution_id": new_execution_id,
            "new_send_at": new_send_at.to_rfc3339()
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        tracing::error!("Failed to update reminder in DB: {}", error_text);
        return Err(Box::new(simple_error::SimpleError::new(format!(
            "Failed to update reminder: {}",
            error_text
        ))));
    }

    tracing::info!("Updated reminder {} in database", reminder_id);
    Ok(())
}

#[tracing::instrument()]
pub async fn function_handler(
    event: LambdaEvent<IncomingMessage>,
) -> Result<OutgoingMessage, Error> {
    let payload = event.payload;

    tracing::info!(
        "Processing reminder for phone: {}, has repeat_config: {}",
        payload.reminder.phone,
        payload.repeat_config.is_some()
    );

    // For recurring reminders, check if the reminder is still active
    if let Some(reminder_id) = payload.reminder_id {
        if !check_reminder_active(reminder_id).await? {
            tracing::info!("Reminder {} has been deleted, skipping", reminder_id);
            return Ok(OutgoingMessage {
                req_id: event.context.request_id,
                msg: "reminder was deleted, skipped".into(),
            });
        }
    }

    // Send the WhatsApp message
    send_msg(&payload.reminder.phone, &payload.reminder.body).await?;

    tracing::info!("Successfully sent reminder to {}", payload.reminder.phone);

    // If this is a recurring reminder, schedule the next occurrence
    if let Some(ref repeat_config) = payload.repeat_config {
        if let (Some(reminder_id), Some(ref timezone)) = (payload.reminder_id, &payload.user_timezone)
        {
            match schedule_next_reminder(&payload, repeat_config, reminder_id, timezone).await {
                Ok(execution_arn) => {
                    tracing::info!("Scheduled next reminder: {}", execution_arn);
                }
                Err(e) => {
                    tracing::error!("Failed to schedule next reminder: {}", e);
                    // Don't fail the whole function, the current message was sent
                }
            }
        }
    }

    Ok(OutgoingMessage {
        req_id: event.context.request_id,
        msg: "reminder was sent".into(),
    })
}
