pub mod blog;
pub mod checkout;
pub mod errors;
pub mod forms;
pub mod middleware;
pub mod oauth;
pub mod pet;
pub mod pet_health;
pub mod pet_note;
pub mod profile;
pub mod reminder;
pub mod routes;
pub mod server;
pub mod templates;
pub mod utils;

use crate::{repo, services};
use csrf::AesGcmCsrfProtection;

pub struct AppState {
    pub csrf_protec: AesGcmCsrfProtection,
    pub repo: repo::ImplAppRepo,
    pub storage_service: services::ImplStorageService,
    pub notification_service: services::ImplNotificationService,
}
