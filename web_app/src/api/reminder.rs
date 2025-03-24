use crate::{config, models, repo, services, utils};
use anyhow::bail;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use log::error;
use serde_json::json;

pub async fn send_verification(phone_number: &str) -> anyhow::Result<()> {
    let totp = utils::totp_client.generate_current();

    if let Ok(otp) = totp {
        let response = utils::request_client
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
                "name": "verify_reminder_phone_es",
                "language": {"code": "es"},
                "components": [
                  {
                    "type": "body",
                    "parameters": [{"type": "text","text": &otp}]
                  },
                  {
                    "type": "button",
                    "sub_type": "url",
                    "index": "0",
                    "parameters": [{"type": "text","text": &otp}]
                  }
                ]
              }
            }))
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(());
        }

        let response = response
            .json::<serde_json::Value>()
            .await
            .unwrap_or_default();

        error!("{response:#?}");
    }

    bail!("error sending otp to phone number")
}

pub async fn validate_otp(otp: &str) -> bool {
    utils::totp_client.check_current(otp).unwrap_or(false)
}

pub async fn add_verified_phone_to_user(
    user_app_id: i64,
    phone: &str,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<()> {
    repo.insert_verified_phone_to_user_app(user_app_id, phone)
        .await
}

pub async fn remove_verified_phone_to_user(
    user_app_id: i64,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<()> {
    repo.set_to_null_verified_phone(user_app_id).await
}

#[derive(Debug)]
pub struct ScheduleReminderInfo {
    pub user_id: i64,
    pub phone_number: String,
    pub when: DateTime<Tz>,
    pub body: String,
}

pub async fn schedule_reminder(
    reminder_info: ScheduleReminderInfo,
    repo: &repo::ImplAppRepo,
    notification_service: &services::ImplNotificationService,
) -> anyhow::Result<()> {
    let reminder = models::reminder::Reminder {
        id: 0,
        user_app_id: reminder_info.user_id,
        body: reminder_info.body.to_string(),
        execution_id: notification_service
            .send_reminder_to_phone_number(&reminder_info)
            .await?,
        notification_type: models::reminder::ReminderNotificationType::WhatsApp,
        user_timezone: reminder_info.when.timezone().name().to_string(),
        send_at: reminder_info.when.to_utc(),
        created_at: Utc::now(),
    };

    repo.insert_user_remider(&reminder).await?;

    Ok(())
}

pub async fn get_scheduled_reminders(
    user_app_id: i64,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<Vec<models::reminder::Reminder>> {
    repo.get_active_user_remiders(user_app_id).await
}

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
    }

    repo.delete_user_reminder(reminder_id, user_id).await
}
