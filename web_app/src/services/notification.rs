use async_trait::async_trait;
use serde_json::json;

use crate::{api, config::APP_CONFIG};

#[derive(Clone)]
pub struct NotificationHandler {
    pub client: aws_sdk_sfn::Client,
}

#[async_trait]
impl crate::services::NotificationService for NotificationHandler {
    async fn send_reminder_to_phone_number(
        &self,
        info: &api::reminder::ScheduleReminderInfo,
    ) -> anyhow::Result<String> {
        let rsp = self
            .client
            .start_execution()
            .state_machine_arn(&APP_CONFIG.aws_sfn_arn_wb_notifications)
            .input(
                json!({
                    "when": info.when.to_rfc3339(),
                    "reminder": {
                        "phone": info.phone_number,
                        "body": info.body
                    }
                })
                .to_string(),
            )
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
