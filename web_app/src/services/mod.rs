pub mod notification;
pub mod storage;

use crate::api;
use async_trait::async_trait;

#[async_trait]
pub trait StorageService {
    async fn save_pic(
        &self,
        user_email: &str,
        file_name: &str,
        body: Vec<u8>,
    ) -> anyhow::Result<()>;

    async fn get_pic_as_bytes(&self, file_name: &str) -> anyhow::Result<Vec<u8>>;
}

#[async_trait]
pub trait NotificationService {
    async fn send_reminder_to_phone_number(
        &self,
        info: &api::reminder::ScheduleReminderInfo,
    ) -> anyhow::Result<String>;

    async fn cancel_reminder_to_phone_number(&self, execution_id: &str) -> anyhow::Result<()>;
}

pub type ImplStorageService = Box<dyn StorageService>;
pub type ImplNotificationService = Box<dyn NotificationService>;
