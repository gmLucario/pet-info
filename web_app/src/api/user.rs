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
/// Contacts information is global for all user pets. `pet_external_id` is for
/// public info, mainly, cause `pet_external_id` is the entry point of data
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::MockAppRepo;
    use chrono::Utc;
    use mockall::predicate::*;
    use uuid::Uuid;

    fn create_test_user(id: i64, email: &str) -> models::user_app::User {
        models::user_app::User {
            id,
            email: email.to_string(),
            phone_reminder: None,
            account_role: models::user_app::AccountRole::User,
            is_subscribed: false,
            is_enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn create_test_owner_contact(
        id: i64,
        user_app_id: i64,
        name: &str,
        value: &str,
    ) -> models::user_app::OwnerContact {
        models::user_app::OwnerContact {
            id,
            user_app_id,
            full_name: name.to_string(),
            contact_value: value.to_string(),
            created_at: Utc::now(),
        }
    }

    fn create_test_payment(user_id: i64) -> models::payment::Payment {
        models::payment::Payment {
            user_id,
            mp_paym_id: 123456,
            payment_idempotency_h: "test_hash".to_string(),
            transaction_amount: "9.99".to_string(),
            installments: 1,
            payment_method_id: "visa".to_string(),
            issuer_id: "123".to_string(),
            status: models::payment::PaymentStatus::Approved,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[ntex::test]
    async fn test_get_or_create_app_user_by_email_existing_user() {
        let expected_email = "test@example.com";

        let mut mock_repo = MockAppRepo::new();
        mock_repo
            .expect_get_user_app_by_email()
            .with(eq(expected_email))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Ok(Some(create_test_user(1, expected_email))) })
            });
        let mock_repo: Box<dyn repo::AppRepo> = Box::new(mock_repo);

        let result = get_or_create_app_user_by_email(&mock_repo, expected_email).await;

        assert!(result.is_ok_and(|u| u.email == expected_email))
    }

    #[ntex::test]
    async fn test_get_or_create_app_user_by_email_new_user() {
        let new_user_id = 42;

        let mut mock_repo = MockAppRepo::new();
        mock_repo
            .expect_get_user_app_by_email()
            .with(eq("new@example.com"))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(None) }));
        mock_repo
            .expect_insert_user_app()
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(new_user_id) }));
        let mock_repo: Box<dyn repo::AppRepo> = Box::new(mock_repo);

        let result = get_or_create_app_user_by_email(&mock_repo, "new@example.com").await;

        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.email, "new@example.com");
        assert_eq!(result.account_role, models::user_app::AccountRole::User);
        assert!(!result.is_subscribed && result.is_enabled);
    }

    #[ntex::test]
    async fn test_get_user_add_pet_balance() {
        let mut mock_repo = MockAppRepo::new();
        let user_id = 1;
        let expected_balance = 5;

        mock_repo
            .expect_get_pet_balance()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(expected_balance) }));

        let mock_repo: Box<dyn repo::AppRepo> = Box::new(mock_repo);
        let result = get_user_add_pet_balance(&mock_repo, user_id).await;

        assert!(result.is_ok_and(|balance| balance.eq(&expected_balance)));
    }

    #[ntex::test]
    async fn test_get_owner_contacts_with_pet_external_id_cause_user_id_zero() {
        let mut mock_repo = MockAppRepo::new();
        let pet_external_id = Uuid::new_v4();
        let expected_contacts = vec![create_test_owner_contact(1, 0, "Phone", "555-1234")];

        mock_repo
            .expect_get_pet_owner_contacts()
            .with(eq(pet_external_id))
            .times(1)
            .returning(move |_| {
                let contacts = expected_contacts.clone();
                Box::pin(async move { Ok(contacts) })
            });

        let mock_repo: Box<dyn repo::AppRepo> = Box::new(mock_repo);
        let result = get_owner_contacts(0, Some(pet_external_id), &mock_repo).await;

        assert!(result.is_ok_and(|contacts| {
            contacts.len() == 1
                && contacts[0].full_name == "Phone"
                && contacts[0].contact_value == "555-1234"
        }));
    }

    #[ntex::test]
    async fn test_get_owner_contacts_with_pet_external_id_without_user_id() {
        let mut mock_repo = MockAppRepo::new();
        let user_id = 1;
        let expected_contacts = vec![
            create_test_owner_contact(1, user_id, "Phone", "555-1234"),
            create_test_owner_contact(2, user_id, "Email", "test@example.com"),
        ];

        mock_repo
            .expect_get_owner_contacts()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| {
                let contacts = expected_contacts.clone();
                Box::pin(async move { Ok(contacts) })
            });

        let mock_repo: Box<dyn repo::AppRepo> = Box::new(mock_repo);
        let result = get_owner_contacts(user_id, None, &mock_repo).await;

        assert!(result.is_ok_and(|contacts| contacts.len() == 2));
    }

    #[ntex::test]
    async fn test_owner_contact_request_fields_are_valid() {
        let valid_request = OwnerContactRequest {
            contact_name: "Phone".into(),
            contact_value: "555-1234".into(),
        };
        assert!(valid_request.fields_are_valid());

        let invalid_request_empty_name = OwnerContactRequest {
            contact_name: "".into(),
            contact_value: "555-1234".into(),
        };
        assert!(!invalid_request_empty_name.fields_are_valid());

        let invalid_request_whitespace = OwnerContactRequest {
            contact_name: "   ".into(),
            contact_value: "555-1234".into(),
        };
        assert!(!invalid_request_whitespace.fields_are_valid());

        let invalid_request_empty_value = OwnerContactRequest {
            contact_name: "Phone".into(),
            contact_value: "".into(),
        };
        assert!(!invalid_request_empty_value.fields_are_valid());
    }

    #[ntex::test]
    async fn test_add_owner_contact() {
        let mut mock_repo = MockAppRepo::new();
        let user_id = 1;
        let request = OwnerContactRequest {
            contact_name: "Phone".to_string(),
            contact_value: "555-1234".to_string(),
        };
        let expected_contact = create_test_owner_contact(1, user_id, "Phone", "555-1234");

        mock_repo
            .expect_insert_owner_contact()
            .with(
                eq(user_id),
                eq("Phone".to_string()),
                eq("555-1234".to_string()),
            )
            .times(1)
            .returning(move |_, _, _| {
                let contact = expected_contact.clone();
                Box::pin(async move { Ok(contact) })
            });

        let mock_repo: Box<dyn repo::AppRepo> = Box::new(mock_repo);
        let result = add_owner_contact(user_id, &request, &mock_repo).await;

        assert!(result.is_ok_and(|contact| {
            contact.full_name == "Phone"
                && contact.contact_value == "555-1234"
                && contact.user_app_id == user_id
        }));
    }

    #[ntex::test]
    async fn test_delete_owner_contact() {
        let mut mock_repo = MockAppRepo::new();
        let user_id = 1;
        let contact_id = 1;

        mock_repo
            .expect_delete_owner_contact()
            .with(eq(user_id), eq(contact_id))
            .times(1)
            .returning(|_, _| Box::pin(async move { Ok(()) }));

        let mock_repo: Box<dyn repo::AppRepo> = Box::new(mock_repo);
        let result = delete_owner_contact(user_id, contact_id, &mock_repo).await;

        assert!(result.is_ok());
    }

    #[ntex::test]
    async fn test_get_payments() {
        let mut mock_repo = MockAppRepo::new();
        let user_id = 1;
        let expected_payments = vec![create_test_payment(user_id)];

        mock_repo
            .expect_get_user_payments()
            .with(eq(user_id), eq(None))
            .times(1)
            .returning(move |_, _| {
                let payments = expected_payments.clone();
                Box::pin(async move { Ok(payments) })
            });

        let mock_repo: Box<dyn repo::AppRepo> = Box::new(mock_repo);
        let result = get_payments(user_id, &mock_repo).await;

        assert!(result.is_ok_and(|pymnts| {
            pymnts.len() == 1
                && pymnts[0].user_id == user_id
                && pymnts[0]
                    .status
                    .eq(&models::payment::PaymentStatus::Approved)
        }))
    }

    #[ntex::test]
    async fn test_delete_user_data() {
        let mut mock_repo = MockAppRepo::new();
        let user_id = 1;

        mock_repo
            .expect_remove_user_app_data()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(()) }));

        let mock_repo: Box<dyn repo::AppRepo> = Box::new(mock_repo);
        let result = delete_user_data(user_id, &mock_repo).await;

        assert!(result.is_ok());
    }

    #[ntex::test]
    async fn test_reactivate_account() {
        let mut mock_repo = MockAppRepo::new();
        let user_id = 1;

        mock_repo
            .expect_set_user_as_active()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(()) }));

        let mock_repo: Box<dyn repo::AppRepo> = Box::new(mock_repo);
        let result = reactivate_account(user_id, &mock_repo).await;

        assert!(result.is_ok());
    }

    #[ntex::test]
    async fn test_get_or_create_app_user_by_email_repository_error() {
        let mut mock_repo = MockAppRepo::new();

        mock_repo
            .expect_get_user_app_by_email()
            .with(eq("test@example.com"))
            .times(1)
            .returning(|_| {
                Box::pin(async move { Err(anyhow::anyhow!("Database connection error")) })
            });

        let boxed_repo: Box<dyn repo::AppRepo> = Box::new(mock_repo);
        let result = get_or_create_app_user_by_email(&boxed_repo, "test@example.com").await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Database connection error")
        );
    }

    #[ntex::test]
    async fn test_get_user_add_pet_balance_error() {
        let mut mock_repo = MockAppRepo::new();
        let user_id = 1;

        mock_repo
            .expect_get_pet_balance()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Box::pin(async move { Err(anyhow::anyhow!("User not found")) }));

        let boxed_repo: Box<dyn repo::AppRepo> = Box::new(mock_repo);
        let result = get_user_add_pet_balance(&boxed_repo, user_id).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("User not found"));
    }
}
