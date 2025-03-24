#[derive(serde::Deserialize, Debug)]
pub struct UserReminderForm {
    pub when: String,
    pub body: String,
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
