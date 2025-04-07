use crate::{models, repo};
use uuid::Uuid;

pub async fn get_or_create_app_user_by_email(
    repo: &repo::ImplAppRepo,
    email: &str,
) -> anyhow::Result<models::user_app::User> {
    if let Some(user) = repo.get_user_app_by_email(email).await? {
        return Ok(user);
    }

    let mut user = models::user_app::User::create_default_from_email(email);
    user.id = repo.save_user_app(&user).await?;

    Ok(user)
}

pub async fn get_owner_contacts(
    user_app_id: i64,
    pet_external_id: Option<Uuid>,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<Vec<models::user_app::OwnerContact>> {
    if let Some(pet_external_id) = pet_external_id {
        return repo.get_pet_owner_contacts(pet_external_id).await;
    }

    repo.get_owner_contacts(user_app_id).await
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct OwnerContactRequest {
    pub contact_name: String,
    pub contact_value: String,
}

impl OwnerContactRequest {
    pub fn fields_are_valid(&self) -> bool {
        !self
            .contact_name
            .split_whitespace()
            .collect::<String>()
            .is_empty()
            && !self
                .contact_value
                .split_whitespace()
                .collect::<String>()
                .is_empty()
    }
}

pub async fn add_owner_contact(
    user_app_id: i64,
    request: &OwnerContactRequest,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<models::user_app::OwnerContact> {
    repo.insert_owner_contact(
        user_app_id,
        request.contact_name.to_string(),
        request.contact_value.to_string(),
    )
    .await
}

pub async fn delete_owner_contact(
    user_app_id: i64,
    contact_id: i64,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<()> {
    repo.delete_owner_contact(user_app_id, contact_id).await
}

pub async fn get_payments(
    user_app_id: i64,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<Vec<models::payment::Payment>> {
    repo.get_user_payments(user_app_id).await
}

pub async fn delete_user_data(user_app_id: i64, repo: &repo::ImplAppRepo) -> anyhow::Result<()> {
    repo.remove_user_app_data(user_app_id).await
}

pub async fn reactivate_account(user_app_id: i64, repo: &repo::ImplAppRepo) -> anyhow::Result<()> {
    repo.set_user_as_active(user_app_id).await
}
