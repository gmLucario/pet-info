pub mod sqlite;
pub mod sqlite_queries;

use crate::models;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait AppRepo {
    async fn remove_user_app_data(&self, user_id: i64) -> anyhow::Result<()>;

    async fn set_user_as_active(&self, user_id: i64) -> anyhow::Result<()>;

    async fn get_user_app_by_email(
        &self,
        email: &str,
    ) -> anyhow::Result<Option<models::user_app::User>>;

    async fn insert_verified_phone_to_user_app(
        &self,
        user_app_id: i64,
        phone: &str,
    ) -> anyhow::Result<()>;

    async fn set_to_null_verified_phone(&self, user_app_id: i64) -> anyhow::Result<()>;

    async fn save_user_app(&self, app_user: &models::user_app::User) -> anyhow::Result<i64>;

    async fn get_user_payments(
        &self,
        user_id: i64,
    ) -> anyhow::Result<Vec<models::payment::Payment>>;

    async fn save_subs_payment(&self, payment: &models::payment::Payment) -> anyhow::Result<i64>;

    async fn set_user_as_subscribed(&self, user_id: i64) -> anyhow::Result<()>;

    async fn save_or_update_pet(&self, pet: &models::pet::Pet, insert: bool)
        -> anyhow::Result<i64>;

    async fn delete_pet(&self, pet_id: i64, user_id: i64) -> anyhow::Result<()>;

    async fn get_all_pets_user_id(&self, user_id: i64) -> anyhow::Result<Vec<models::pet::Pet>>;

    async fn get_pet_by_external_id(
        &self,
        pet_external_id: Uuid,
    ) -> anyhow::Result<models::pet::Pet>;

    async fn get_pet_pic_path_by_external_id(
        &self,
        pet_external_id: Uuid,
    ) -> anyhow::Result<Option<String>>;

    async fn get_pet_by_id(&self, pet_id: i64, user_id: i64) -> anyhow::Result<models::pet::Pet>;

    async fn get_pet_weights(
        &self,
        pet_external_id: Uuid,
        user_id: Option<i64>,
    ) -> anyhow::Result<Vec<models::pet::PetWeight>>;

    async fn get_pet_health_records(
        &self,
        pet_external_id: Uuid,
        user_id: Option<i64>,
        health_type: models::pet::PetHealthType,
    ) -> anyhow::Result<Vec<models::pet::PetHealth>>;

    async fn insert_vaccine_to(
        &self,
        pet_external_id: Uuid,
        user_id: i64,
        desc: String,
        date: chrono::NaiveDate,
    ) -> anyhow::Result<models::pet::PetHealth>;

    async fn delete_vaccine(
        &self,
        pet_external_id: Uuid,
        user_id: i64,
        vaccine_id: i64,
    ) -> anyhow::Result<()>;

    async fn insert_deworm_to(
        &self,
        pet_external_id: Uuid,
        user_id: i64,
        desc: String,
        date: chrono::NaiveDate,
    ) -> anyhow::Result<models::pet::PetHealth>;

    async fn delete_deworm(
        &self,
        pet_external_id: Uuid,
        user_id: i64,
        deworm_id: i64,
    ) -> anyhow::Result<()>;

    async fn insert_pet_weight(
        &self,
        pet_external_id: Uuid,
        user_id: i64,
        weight: f64,
        date: chrono::NaiveDate,
    ) -> anyhow::Result<models::pet::PetWeight>;

    async fn delete_pet_weight(
        &self,
        pet_external_id: Uuid,
        user_id: i64,
        weight_id: i64,
    ) -> anyhow::Result<()>;

    async fn get_pet_owner_contacts(
        &self,
        pet_external_id: Uuid,
    ) -> anyhow::Result<Vec<models::user_app::OwnerContact>>;

    async fn get_owner_contacts(
        &self,
        user_id: i64,
    ) -> anyhow::Result<Vec<models::user_app::OwnerContact>>;

    async fn insert_owner_contact(
        &self,
        user_id: i64,
        desc: String,
        contact: String,
    ) -> anyhow::Result<models::user_app::OwnerContact>;

    async fn delete_owner_contact(&self, user_id: i64, contact_id: i64) -> anyhow::Result<()>;

    async fn insert_new_pet_note(
        &self,
        user_id: i64,
        note: &models::pet::PetNote,
    ) -> anyhow::Result<i64>;

    async fn get_pet_notes(
        &self,
        user_id: i64,
        pet_id: i64,
    ) -> anyhow::Result<Vec<models::pet::PetNote>>;

    async fn get_reminder_execution_id(
        &self,
        user_id: i64,
        reminder_id: i64,
    ) -> anyhow::Result<Option<String>>;

    async fn delete_pet_note(&self, pet_id: i64, user_id: i64, note_id: i64) -> anyhow::Result<()>;

    async fn get_active_user_remiders(
        &self,
        user_id: i64,
    ) -> anyhow::Result<Vec<models::reminder::Reminder>>;

    async fn insert_user_remider(
        &self,
        reminder: &models::reminder::Reminder,
    ) -> anyhow::Result<i64>;

    async fn delete_user_reminder(&self, reminder_id: i64, user_id: i64) -> anyhow::Result<()>;
}

pub type ImplAppRepo = Box<dyn AppRepo>;
