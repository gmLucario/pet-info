use chrono::{DateTime, Utc};
use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::consts;

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

#[derive(Serialize, Debug, Deserialize)]
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
        self.end_free_trial()
            .map(|end_free_trial| end_free_trial.date_naive() > Utc::now().date_naive())
            .unwrap_or(self.is_subscribed && self.is_enabled)
    }

    pub fn end_free_trial(&self) -> Option<DateTime<Utc>> {
        if !self.is_subscribed {
            return Some(self.created_at + chrono::Duration::days(consts::DAYS_FREE_TRIAL));
        }
        None
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

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct OwnerContact {
    pub id: i64,
    pub user_app_id: i64,
    pub full_name: String,
    pub contact_value: String,
    pub created_at: DateTime<Utc>,
}
