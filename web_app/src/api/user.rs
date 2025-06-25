//! # User API Module
//!
//! This module handles user management operations including user creation,
//! authentication, contact management, and user profile operations.

use crate::{metric, models, repo};
use uuid::Uuid;

/// Gets an existing user by email or creates a new one if not found.
///
/// This function implements a get-or-create pattern for user management.
/// It first attempts to find an existing user with the given email address,
/// and if none is found, creates a new user with default settings.
///
/// # Arguments
/// * `repo` - Repository instance for database operations
/// * `email` - Email address to look up or create user for
///
/// # Returns
/// * `anyhow::Result<models::user_app::User>` - The existing or newly created user
///
/// # Process
/// 1. Search for existing user by email
/// 2. Return existing user if found
/// 3. Create new user with default settings if not found
/// 4. Record user creation metrics
///
/// # Errors
/// Returns an error if database operations fail during user lookup or creation.
pub async fn get_or_create_app_user_by_email(
    repo: &repo::ImplAppRepo,
    email: &str,
) -> anyhow::Result<models::user_app::User> {
    if let Some(user) = repo.get_user_app_by_email(email).await? {
        return Ok(user);
    }

    let mut user = models::user_app::User::create_default_from_email(email);
    user.id = repo.insert_user_app(&user).await?;

    metric::incr_user_action_statds("create_user");
    Ok(user)
}

/// Retrieves the pet balance for a user.
///
/// Gets the number of pet slots available for the user to create new pets.
/// This balance is typically increased through subscription payments.
///
/// # Arguments
/// * `repo` - Repository instance for database operations
/// * `user_id` - ID of the user to get balance for
///
/// # Returns
/// * `anyhow::Result<u32>` - Number of available pet slots
pub async fn get_user_add_pet_balance(
    repo: &repo::ImplAppRepo,
    user_id: i64,
) -> anyhow::Result<u32> {
    repo.get_pet_balance(user_id).await
}

/// Retrieves contact information for a user or specific pet.
///
/// Gets contact information that can be displayed on public pet profiles.
/// If a pet external ID is provided, returns contacts specific to that pet;
/// otherwise returns all user contacts.
///
/// # Arguments
/// * `user_app_id` - ID of the user who owns the contacts
/// * `pet_external_id` - Optional pet ID to get pet-specific contacts
/// * `repo` - Repository instance for database operations
///
/// # Returns
/// * `anyhow::Result<Vec<models::user_app::OwnerContact>>` - List of contact information
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

/// Request structure for creating or updating owner contact information.
///
/// Contains the contact details submitted by users for display on their
/// pet profiles. This information allows people who find a pet to contact
/// the owner.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct OwnerContactRequest {
    /// Display name for the contact method (e.g., "Phone", "Email")
    pub contact_name: String,
    /// Contact value (e.g., phone number, email address)
    pub contact_value: String,
}

impl OwnerContactRequest {
    /// Validates that both contact fields contain non-empty content.
    ///
    /// Checks that both the contact name and value contain actual content
    /// (not just whitespace) to ensure valid contact information.
    ///
    /// # Returns
    /// * `bool` - True if both fields are valid, false otherwise
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

/// Adds a new contact method to a user's profile.
///
/// Creates a new contact entry that will be displayed on the user's
/// pet profiles, allowing people to contact the owner if needed.
///
/// # Arguments
/// * `user_app_id` - ID of the user to add contact for
/// * `request` - Contact information to add
/// * `repo` - Repository instance for database operations
///
/// # Returns
/// * `anyhow::Result<models::user_app::OwnerContact>` - The created contact record
///
/// # Validation
/// The request should be validated using `fields_are_valid()` before calling this function.
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

/// Removes a contact method from a user's profile.
///
/// Deletes a specific contact entry from the user's profile. This will
/// remove it from all pet profiles associated with the user.
///
/// # Arguments
/// * `user_app_id` - ID of the user who owns the contact
/// * `contact_id` - ID of the contact to delete
/// * `repo` - Repository instance for database operations
///
/// # Returns
/// * `anyhow::Result<()>` - Success confirmation or error details
pub async fn delete_owner_contact(
    user_app_id: i64,
    contact_id: i64,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<()> {
    repo.delete_owner_contact(user_app_id, contact_id).await
}

/// Retrieves payment history for a user.
///
/// Gets all payment records associated with the user, including
/// subscription payments and their status.
///
/// # Arguments
/// * `user_app_id` - ID of the user to get payments for
/// * `repo` - Repository instance for database operations
///
/// # Returns
/// * `anyhow::Result<Vec<models::payment::Payment>>` - List of user payments
pub async fn get_payments(
    user_app_id: i64,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<Vec<models::payment::Payment>> {
    repo.get_user_payments(user_app_id, None).await
}

/// Deletes all user data and deactivates the account.
///
/// Removes all user data including pets, payments, contacts, and reminders.
/// This is typically used for GDPR compliance and account deletion requests.
///
/// # Arguments
/// * `user_app_id` - ID of the user to delete data for
/// * `repo` - Repository instance for database operations
///
/// # Returns
/// * `anyhow::Result<()>` - Success confirmation or error details
///
/// # Warning
/// This operation is irreversible and will permanently delete all user data.
pub async fn delete_user_data(user_app_id: i64, repo: &repo::ImplAppRepo) -> anyhow::Result<()> {
    repo.remove_user_app_data(user_app_id).await
}

/// Reactivates a deactivated user account.
///
/// Sets the user status back to active, allowing them to use the
/// application again. This is used when users want to restore
/// previously deactivated accounts.
///
/// # Arguments
/// * `user_app_id` - ID of the user to reactivate
/// * `repo` - Repository instance for database operations
///
/// # Returns
/// * `anyhow::Result<()>` - Success confirmation or error details
pub async fn reactivate_account(user_app_id: i64, repo: &repo::ImplAppRepo) -> anyhow::Result<()> {
    repo.set_user_as_active(user_app_id).await
}
