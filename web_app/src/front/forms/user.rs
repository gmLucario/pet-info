use chrono::NaiveDateTime;
use serde::{Deserialize, Deserializer};

use crate::consts;

#[derive(serde::Deserialize, Debug)]
pub struct UserReminderForm {
    #[serde(deserialize_with = "deserialize_when_user_input")]
    pub when: NaiveDateTime,
    pub body: String,
}

fn deserialize_when_user_input<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    NaiveDateTime::parse_from_str(&buf, consts::DATETIME_LOCAL_INPUT_FORMAT)
        .map_err(serde::de::Error::custom)
}

#[derive(serde::Deserialize, Debug)]
pub struct ReminderPhoneToVerify {
    pub country_phone_code: u32,
    pub reminders_phone: usize,
}

#[derive(serde::Deserialize, Debug)]
pub struct ReminderPhoneOtp {
    pub otp_value: String,
}
