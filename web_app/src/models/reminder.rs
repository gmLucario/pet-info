use chrono::{DateTime, Utc};
use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Display, Clone, Default, Deserialize, Serialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum ReminderNotificationType {
    #[default]
    #[display("whatsapp")]
    #[serde(alias = "whatsapp", rename(serialize = "whatsapp"))]
    WhatsApp,
}

#[derive(Debug, Display, Clone, Deserialize, Serialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum RepeatType {
    #[display("daily")]
    #[serde(alias = "daily", rename(serialize = "daily"))]
    Daily,
    #[display("weekly")]
    #[serde(alias = "weekly", rename(serialize = "weekly"))]
    Weekly,
    #[display("monthly")]
    #[serde(alias = "monthly", rename(serialize = "monthly"))]
    Monthly,
    #[display("yearly")]
    #[serde(alias = "yearly", rename(serialize = "yearly"))]
    Yearly,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RepeatConfig {
    pub repeat_type: RepeatType,
    pub repeat_interval: i32,
}

impl RepeatConfig {
    /// Returns the Spanish summary for the repeat configuration
    pub fn summary_spanish(&self) -> String {
        match (&self.repeat_type, self.repeat_interval) {
            (RepeatType::Daily, 1) => "Se repite diariamente".to_string(),
            (RepeatType::Daily, n) => format!("Se repite cada {} días", n),
            (RepeatType::Weekly, 1) => "Se repite semanalmente".to_string(),
            (RepeatType::Weekly, n) => format!("Se repite cada {} semanas", n),
            (RepeatType::Monthly, 1) => "Se repite mensualmente".to_string(),
            (RepeatType::Monthly, n) => format!("Se repite cada {} meses", n),
            (RepeatType::Yearly, 1) => "Se repite anualmente".to_string(),
            (RepeatType::Yearly, n) => format!("Se repite cada {} años", n),
        }
    }
}

/// Database row structure (maps directly to DB columns)
#[derive(sqlx::FromRow)]
pub struct ReminderRow {
    pub id: i64,
    pub user_app_id: i64,
    pub body: String,
    pub execution_id: String,
    pub notification_type: ReminderNotificationType,
    pub send_at: DateTime<Utc>,
    pub user_timezone: String,
    pub created_at: DateTime<Utc>,
    pub repeat_type: Option<RepeatType>,
    pub repeat_interval: Option<i32>,
}

/// Application struct with RepeatConfig
#[derive(Default, Serialize)]
pub struct Reminder {
    pub id: i64,
    pub user_app_id: i64,
    pub body: String,
    pub execution_id: String,
    pub notification_type: ReminderNotificationType,
    pub send_at: DateTime<Utc>,
    pub user_timezone: String,
    pub created_at: DateTime<Utc>,
    pub repeat_config: Option<RepeatConfig>,
}

impl From<ReminderRow> for Reminder {
    fn from(row: ReminderRow) -> Self {
        let repeat_config = match (row.repeat_type, row.repeat_interval) {
            (Some(rt), Some(ri)) => Some(RepeatConfig {
                repeat_type: rt,
                repeat_interval: ri,
            }),
            _ => None,
        };

        Reminder {
            id: row.id,
            user_app_id: row.user_app_id,
            body: row.body,
            execution_id: row.execution_id,
            notification_type: row.notification_type,
            send_at: row.send_at,
            user_timezone: row.user_timezone,
            created_at: row.created_at,
            repeat_config,
        }
    }
}
