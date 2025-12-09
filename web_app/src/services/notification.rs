use async_trait::async_trait;
use aws_sdk_sfn::types::ExecutionStatus;
use chrono::Utc;
use serde_json::json;

use crate::{api, config};
use anyhow::Context;

#[derive(Clone)]
pub struct NotificationHandler {
    pub client: aws_sdk_sfn::Client,
}

/// Generates a unique execution name for a reminder.
/// Format: `reminder-{reminder_id}-{timestamp}`
fn generate_execution_name(reminder_id: i64) -> String {
    format!("reminder-{}-{}", reminder_id, Utc::now().timestamp_millis())
}

/// Returns the execution name prefix for a reminder.
/// Used for listing executions to cancel.
fn execution_name_prefix(reminder_id: i64) -> String {
    format!("reminder-{}-", reminder_id)
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
            },
            "reminder_id": reminder_id,
            "user_timezone": info.when.timezone().name()
        });

        // Include repeat config if present (for recurring reminders)
        if let Some(ref config) = info.repeat_config {
            payload["repeat_config"] = json!({
                "repeat_type": config.repeat_type.to_string(),
                "repeat_interval": config.repeat_interval
            });
        }

        let state_machine_arn = &config::APP_CONFIG
            .get()
            .context("failed to get app config")?
            .aws_sfn_arn_wb_notifications;

        let rsp = self
            .client
            .start_execution()
            .state_machine_arn(state_machine_arn)
            .name(generate_execution_name(reminder_id))
            .input(payload.to_string())
            .send()
            .await?;

        Ok(rsp.execution_arn)
    }

    async fn cancel_reminder_executions(&self, reminder_id: i64) -> anyhow::Result<()> {
        let state_machine_arn = &config::APP_CONFIG
            .get()
            .context("failed to get app config")?
            .aws_sfn_arn_wb_notifications;

        let prefix = execution_name_prefix(reminder_id);

        // List all RUNNING executions for this state machine
        let mut paginator = self
            .client
            .list_executions()
            .state_machine_arn(state_machine_arn)
            .status_filter(ExecutionStatus::Running)
            .into_paginator()
            .send();

        // Find and stop all executions matching our reminder prefix
        while let Some(page) = paginator.next().await {
            let page = page?;
            for execution in page.executions() {
                if let Some(name) = execution.name() {
                    if name.starts_with(&prefix) {
                        tracing::info!(
                            "Stopping execution {} for reminder {}",
                            execution.execution_arn(),
                            reminder_id
                        );
                        // Best effort - don't fail if one stop fails
                        let _ = self
                            .client
                            .stop_execution()
                            .execution_arn(execution.execution_arn())
                            .send()
                            .await;
                    }
                }
            }
        }

        Ok(())
    }
}
