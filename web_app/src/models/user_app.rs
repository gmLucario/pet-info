use chrono::{DateTime, Utc};
use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Display, Default)]
pub enum AccountRole {
    #[serde(rename = "user")]
    #[display("user")]
    User,
    #[serde(rename = "staff")]
    #[display("staff")]
    Staff,
    #[default]
    #[serde(rename = "unknown")]
    #[display("unknown")]
    Unknown,
}

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub phone_reminder: Option<String>,
    pub account_role: AccountRole,
    pub is_subscribed: bool,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn can_access_service(&self) -> bool {
        self.is_subscribed && self.is_enabled
    }

    pub fn create_default_from_email(email: &str) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            email: email.to_string(),
            phone_reminder: None,
            account_role: AccountRole::User,
            is_subscribed: false,
            is_enabled: true,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(serde::Serialize, sqlx::FromRow, Clone)]
pub struct OwnerContact {
    pub id: i64,
    pub user_app_id: i64,
    pub full_name: String,
    pub contact_value: String,
    pub created_at: DateTime<Utc>,
}
