use async_trait::async_trait;
use serde_json::json;

use crate::{api, config};
use anyhow::Context;

#[derive(Clone)]
pub struct NotificationHandler {
    pub client: aws_sdk_sfn::Client,
}

#[async_trait]
impl crate::services::NotificationService for NotificationHandler {
    async fn send_reminder_to_phone_number(
        &self,
        info: &api::reminder::ScheduleReminderInfo,
        reminder_id: i64,
    ) -> anyhow::Result<String> {
        let mut payload = json!({
            "when": info.when.to_rfc3339(),
            "reminder": {
                "phone": info.phone_number,
                "body": info.body
            }
        });

        // Include repeat config if present (for recurring reminders)
        if let Some(ref config) = info.repeat_config {
            payload["repeat_config"] = json!({
                "repeat_type": config.repeat_type.to_string(),
                "repeat_interval": config.repeat_interval
            });
            payload["reminder_id"] = json!(reminder_id);
            payload["user_timezone"] = json!(info.when.timezone().name());
        }

        let rsp = self
            .client
            .start_execution()
            .state_machine_arn(
                &config::APP_CONFIG
                    .get()
                    .context("failed to get app config")?
                    .aws_sfn_arn_wb_notifications,
            )
            .input(payload.to_string())
            .send()
            .await?;

        Ok(rsp.execution_arn)
    }

    async fn cancel_reminder_to_phone_number(&self, execution_id: &str) -> anyhow::Result<()> {
        self.client
            .stop_execution()
            .execution_arn(execution_id)
            .send()
            .await?;

        Ok(())
    }
}
