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

#[derive(Default, Serialize, sqlx::FromRow)]
pub struct Reminder {
    pub id: i64,
    pub user_app_id: i64,
    pub body: String,
    pub execution_id: String,
    pub notification_type: ReminderNotificationType,
    pub send_at: DateTime<Utc>,
    pub user_timezone: String,
    pub created_at: DateTime<Utc>,
}
