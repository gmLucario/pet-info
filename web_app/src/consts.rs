use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub const CSRF_TOKEN_COOKIE_NAME: &str = "csrf_token";
pub const CSRF_STATE_COOKIE_NAME: &str = "csrf_state";
pub const REDIRECT_TO_COOKIE_NAME: &str = "redirect_to";
pub const OTP_PHONE_COOKIE_NAME: &str = "otp_phone_value";
pub const GOOGLE_ENDPOINT_USER_INFO: &str = "https://openidconnect.googleapis.com/v1/userinfo";
pub const GOOGLE_ENDPOINT_REVOKE_TOKEN: &str = "https://oauth2.googleapis.com/revoke";
pub const ADD_PET_PRICE: Decimal = dec!(100.00);
pub const PIC_PET_MAX_SIZE_BYTES: usize = 6_000_000;

/// Apple Wallet pass thumbnail dimensions for @2x Retina displays.
/// Based on Apple's specification: 90x90 points = 180x180 pixels at @2x.
/// This size provides optimal quality on most iOS devices while keeping file size minimal.
/// Reference: https://developer.apple.com/library/archive/documentation/UserExperience/Conceptual/PassKit_PG/Creating.html
pub const PKPASS_THUMBNAIL_SIZE_PX: u32 = 180;

pub const S3_MAIN_BUCKET_NAME: &str = "pet-info-app-storage";
pub const DATETIME_LOCAL_INPUT_FORMAT: &str = "%Y-%m-%dT%H:%M";

pub const ACCEPTED_IMAGE_EXTENSIONS: [&str; 4] = ["png", "jpeg", "jpg", "heic"];

pub const MAX_AGE_COOKIES: i64 = chrono::TimeDelta::hours(4).num_seconds();
