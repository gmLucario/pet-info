//! Repository module for the pet info application.
//!
//! This module provides the data access layer for the application, defining
//! the `AppRepo` trait that abstracts database operations for pets, users,
//! payments, and other application entities.

pub mod sqlite;
pub mod sqlite_queries;

use crate::models;
use async_trait::async_trait;
use uuid::Uuid;

/// Main repository trait that defines all database operations for the application.
///
/// This trait provides a clean abstraction over the database layer, allowing
/// different implementations (currently SQLite) while maintaining a consistent
/// interface for the application logic.
///
/// All methods return `anyhow::Result` for error handling and are async to
/// support non-blocking database operations.
#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait AppRepo {
    // User Management

    /// Removes all application data associated with a user.
    ///
    /// # Arguments
    /// * `user_id` - The unique identifier of the user
    async fn remove_user_app_data(&self, user_id: i64) -> anyhow::Result<()>;

    /// Sets a user as active in the system.
    ///
    /// # Arguments
    /// * `user_id` - The unique identifier of the user
    async fn set_user_as_active(&self, user_id: i64) -> anyhow::Result<()>;

    /// Retrieves a user by their email address.
    ///
    /// # Arguments
    /// * `email` - The email address to search for
    ///
    /// # Returns
    /// * `Some(User)` if found, `None` if not found
    async fn get_user_app_by_email(
        &self,
        email: &str,
    ) -> anyhow::Result<Option<models::user_app::User>>;

    /// Retrieves a user by their phone number.
    ///
    /// # Arguments
    /// * `phone` - The phone number to search for (WhatsApp ID format)
    ///
    /// # Returns
    /// * `Some(User)` if found, `None` if not found
    async fn get_user_app_by_phone(
        &self,
        phone: &str,
    ) -> anyhow::Result<Option<models::user_app::User>>;

    /// Associates a verified phone number with a user account.
    ///
    /// # Arguments
    /// * `user_app_id` - The user's unique identifier
    /// * `phone` - The verified phone number
    async fn insert_verified_phone_to_user_app(
        &self,
        user_app_id: i64,
        phone: &str,
    ) -> anyhow::Result<()>;

    /// Removes the verified phone number from a user account.
    ///
    /// # Arguments
    /// * `user_app_id` - The user's unique identifier
    async fn set_to_null_verified_phone(&self, user_app_id: i64) -> anyhow::Result<()>;

    /// Creates a new user in the system.
    ///
    /// # Arguments
    /// * `app_user` - The user data to insert
    ///
    /// # Returns
    /// * The newly created user's ID
    async fn insert_user_app(&self, app_user: &models::user_app::User) -> anyhow::Result<i64>;

    /// Checks if a pet's external ID is linked to a user account.
    ///
    /// # Arguments
    /// * `pet_external_id` - The pet's external UUID
    ///
    /// # Returns
    /// * `Some(true)` if linked, `Some(false)` if not linked, `None` if pet doesn't exist
    async fn is_pet_external_id_linked(
        &self,
        pet_external_id: &Uuid,
    ) -> anyhow::Result<Option<bool>>;

    // Payment Management

    /// Retrieves user payments in descending order by date.
    ///
    /// # Arguments
    /// * `user_id` - The user's unique identifier
    /// * `status` - Optional filter by payment status
    ///
    /// # Returns
    /// * Vector of payments sorted by date (newest first)
    async fn get_user_payments(
        &self,
        user_id: i64,
        status: Option<models::payment::PaymentStatus>,
    ) -> anyhow::Result<Vec<models::payment::Payment>>;

    /// Sets the pet balance for a user.
    ///
    /// # Arguments
    /// * `user_id` - The user's unique identifier
    /// * `balance` - The new balance amount
    async fn set_pet_balance(&self, user_id: i64, balance: u32) -> anyhow::Result<()>;

    /// Retrieves the current pet balance for a user.
    ///
    /// # Arguments
    /// * `user_id` - The user's unique identifier
    ///
    /// # Returns
    /// * The current balance amount
    async fn get_pet_balance(&self, user_id: i64) -> anyhow::Result<u32>;

    /// Saves a subscription payment record.
    ///
    /// # Arguments
    /// * `payment` - The payment data to save
    ///
    /// # Returns
    /// * The newly created payment record ID
    async fn save_subs_payment(&self, payment: &models::payment::Payment) -> anyhow::Result<i64>;

    /// Marks a user as having an active subscription.
    ///
    /// # Arguments
    /// * `user_id` - The user's unique identifier
    async fn set_user_as_subscribed(&self, user_id: i64) -> anyhow::Result<()>;

    // Pet Management

    /// Creates a new pet record.
    ///
    /// # Arguments
    /// * `pet` - The pet data to create
    ///
    /// # Returns
    /// * The newly created pet's ID
    async fn save_pet(&self, pet: &models::pet::Pet) -> anyhow::Result<i64>;

    /// Updates an existing pet record.
    ///
    /// # Arguments
    /// * `pet` - The updated pet data
    ///
    /// # Returns
    /// * The updated pet's ID
    async fn update_pet(&self, pet: &models::pet::Pet) -> anyhow::Result<i64>;

    /// Deletes a pet belonging to a specific user.
    ///
    /// # Arguments
    /// * `pet_id` - The pet's unique identifier
    /// * `user_id` - The owner's user ID (for authorization)
    async fn delete_pet(&self, pet_id: i64, user_id: i64) -> anyhow::Result<()>;

    /// Retrieves all pets belonging to a user.
    ///
    /// # Arguments
    /// * `user_id` - The user's unique identifier
    ///
    /// # Returns
    /// * Vector of all pets owned by the user
    async fn get_all_pets_user_id(&self, user_id: i64) -> anyhow::Result<Vec<models::pet::Pet>>;

    /// Retrieves a pet by its external UUID (public identifier).
    ///
    /// # Arguments
    /// * `pet_external_id` - The pet's external UUID
    ///
    /// # Returns
    /// * The pet data
    async fn get_pet_by_external_id(
        &self,
        pet_external_id: Uuid,
    ) -> anyhow::Result<models::pet::Pet>;

    /// Retrieves the file path for a pet's picture.
    ///
    /// # Arguments
    /// * `pet_external_id` - The pet's external UUID
    ///
    /// # Returns
    /// * `Some(path)` if picture exists, `None` if no picture
    async fn get_pet_pic_path_by_external_id(
        &self,
        pet_external_id: Uuid,
    ) -> anyhow::Result<Option<String>>;

    /// Retrieves a pet by its internal ID, ensuring user ownership.
    ///
    /// # Arguments
    /// * `pet_id` - The pet's internal ID
    /// * `user_id` - The owner's user ID (for authorization)
    ///
    /// # Returns
    /// * The pet data if owned by the user
    async fn get_pet_by_id(&self, pet_id: i64, user_id: i64) -> anyhow::Result<models::pet::Pet>;

    // Pet Weights Management

    /// Retrieves all weight records for a specific pet.
    ///
    /// # Arguments
    /// * `pet_external_id` - The pet's external UUID
    /// * `user_id` - Optional user ID for authorization (if provided, verifies ownership)
    ///
    /// # Returns
    /// * Vector of weight records ordered by date
    async fn get_pet_weights(
        &self,
        pet_external_id: Uuid,
        user_id: Option<i64>,
    ) -> anyhow::Result<Vec<models::pet::PetWeight>>;

    // Pet Health Management

    /// Retrieves health records for a specific pet and health type.
    ///
    /// # Arguments
    /// * `pet_external_id` - The pet's external UUID
    /// * `user_id` - Optional user ID for authorization (if provided, verifies ownership)
    /// * `health_type` - The type of health records to retrieve (vaccines, deworming, etc.)
    ///
    /// # Returns
    /// * Vector of health records for the specified type
    async fn get_pet_health_records(
        &self,
        pet_external_id: Uuid,
        user_id: Option<i64>,
        health_type: models::pet::PetHealthType,
    ) -> anyhow::Result<Vec<models::pet::PetHealth>>;

    /// Adds a new vaccination record to a pet.
    ///
    /// # Arguments
    /// * `pet_external_id` - The pet's external UUID
    /// * `user_id` - The owner's user ID (for authorization)
    /// * `desc` - Description of the vaccine (name, type, etc.)
    /// * `date` - Date when the vaccination was administered
    ///
    /// # Returns
    /// * The newly created vaccination health record
    async fn insert_vaccine_to(
        &self,
        pet_external_id: Uuid,
        user_id: i64,
        desc: String,
        date: chrono::NaiveDate,
    ) -> anyhow::Result<models::pet::PetHealth>;

    /// Removes a vaccination record from a pet.
    ///
    /// # Arguments
    /// * `pet_external_id` - The pet's external UUID
    /// * `user_id` - The owner's user ID (for authorization)
    /// * `vaccine_id` - The unique identifier of the vaccination record to delete
    async fn delete_vaccine(
        &self,
        pet_external_id: Uuid,
        user_id: i64,
        vaccine_id: i64,
    ) -> anyhow::Result<()>;

    /// Adds a new deworming record to a pet.
    ///
    /// # Arguments
    /// * `pet_external_id` - The pet's external UUID
    /// * `user_id` - The owner's user ID (for authorization)
    /// * `desc` - Description of the deworming treatment (medication, dosage, etc.)
    /// * `date` - Date when the deworming was administered
    ///
    /// # Returns
    /// * The newly created deworming health record
    async fn insert_deworm_to(
        &self,
        pet_external_id: Uuid,
        user_id: i64,
        desc: String,
        date: chrono::NaiveDate,
    ) -> anyhow::Result<models::pet::PetHealth>;

    /// Removes a deworming record from a pet.
    ///
    /// # Arguments
    /// * `pet_external_id` - The pet's external UUID
    /// * `user_id` - The owner's user ID (for authorization)
    /// * `deworm_id` - The unique identifier of the deworming record to delete
    async fn delete_deworm(
        &self,
        pet_external_id: Uuid,
        user_id: i64,
        deworm_id: i64,
    ) -> anyhow::Result<()>;

    /// Records a new weight measurement for a pet.
    ///
    /// # Arguments
    /// * `pet_external_id` - The pet's external UUID
    /// * `user_id` - The owner's user ID (for authorization)
    /// * `weight` - The pet's weight measurement
    /// * `date` - Date when the weight was recorded
    ///
    /// # Returns
    /// * The newly created weight record
    async fn insert_pet_weight(
        &self,
        pet_external_id: Uuid,
        user_id: i64,
        weight: f64,
        date: chrono::NaiveDate,
    ) -> anyhow::Result<models::pet::PetWeight>;

    /// Removes a weight record from a pet.
    ///
    /// # Arguments
    /// * `pet_external_id` - The pet's external UUID
    /// * `user_id` - The owner's user ID (for authorization)
    /// * `weight_id` - The unique identifier of the weight record to delete
    async fn delete_pet_weight(
        &self,
        pet_external_id: Uuid,
        user_id: i64,
        weight_id: i64,
    ) -> anyhow::Result<()>;

    // Owner Contacts Management

    /// Retrieves contact information for a pet's owner.
    ///
    /// # Arguments
    /// * `pet_external_id` - The pet's external UUID
    ///
    /// # Returns
    /// * Vector of owner contact information associated with the pet
    async fn get_pet_owner_contacts(
        &self,
        pet_external_id: Uuid,
    ) -> anyhow::Result<Vec<models::user_app::OwnerContact>>;

    /// Retrieves all contact information for a specific user.
    ///
    /// # Arguments
    /// * `user_id` - The user's unique identifier
    ///
    /// # Returns
    /// * Vector of all contact information belonging to the user
    async fn get_owner_contacts(
        &self,
        user_id: i64,
    ) -> anyhow::Result<Vec<models::user_app::OwnerContact>>;

    /// Adds a new contact entry for a user.
    ///
    /// # Arguments
    /// * `user_id` - The user's unique identifier
    /// * `desc` - Description of the contact (e.g., "Primary Vet", "Emergency Contact")
    /// * `contact` - The contact information (phone, email, address, etc.)
    ///
    /// # Returns
    /// * The newly created owner contact record
    async fn insert_owner_contact(
        &self,
        user_id: i64,
        desc: String,
        contact: String,
    ) -> anyhow::Result<models::user_app::OwnerContact>;

    /// Removes a contact entry from a user's contact list.
    ///
    /// # Arguments
    /// * `user_id` - The user's unique identifier (for authorization)
    /// * `contact_id` - The unique identifier of the contact to delete
    async fn delete_owner_contact(&self, user_id: i64, contact_id: i64) -> anyhow::Result<()>;

    // Pet Notes Management

    /// Creates a new note for a pet.
    ///
    /// # Arguments
    /// * `user_id` - The owner's user ID (for authorization)
    /// * `note` - The note data to create
    ///
    /// # Returns
    /// * The newly created note's ID
    async fn insert_new_pet_note(
        &self,
        user_id: i64,
        note: &models::pet::PetNote,
    ) -> anyhow::Result<i64>;

    /// Retrieves all notes for a specific pet.
    ///
    /// # Arguments
    /// * `user_id` - The owner's user ID (for authorization)
    /// * `pet_id` - The pet's unique identifier
    ///
    /// # Returns
    /// * Vector of all notes associated with the pet
    async fn get_pet_notes(
        &self,
        user_id: i64,
        pet_id: i64,
    ) -> anyhow::Result<Vec<models::pet::PetNote>>;

    /// Removes a note from a pet.
    ///
    /// # Arguments
    /// * `pet_id` - The pet's unique identifier
    /// * `user_id` - The owner's user ID (for authorization)
    /// * `note_id` - The unique identifier of the note to delete
    async fn delete_pet_note(&self, pet_id: i64, user_id: i64, note_id: i64) -> anyhow::Result<()>;

    // Reminders Management

    /// Retrieves the execution ID for a specific reminder.
    ///
    /// This is used for tracking scheduled reminder executions in external systems.
    ///
    /// # Arguments
    /// * `user_id` - The user's unique identifier (for authorization)
    /// * `reminder_id` - The reminder's unique identifier
    ///
    /// # Returns
    /// * `Some(execution_id)` if the reminder has an execution ID, `None` otherwise
    async fn get_reminder_execution_id(
        &self,
        user_id: i64,
        reminder_id: i64,
    ) -> anyhow::Result<Option<String>>;

    /// Retrieves all active reminders for a user.
    ///
    /// # Arguments
    /// * `user_id` - The user's unique identifier
    ///
    /// # Returns
    /// * Vector of all active reminders belonging to the user
    async fn get_active_user_remiders(
        &self,
        user_id: i64,
    ) -> anyhow::Result<Vec<models::reminder::Reminder>>;

    /// Creates a new reminder for a user.
    ///
    /// # Arguments
    /// * `reminder` - The reminder data to create
    ///
    /// # Returns
    /// * The newly created reminder's ID
    async fn insert_user_remider(
        &self,
        reminder: &models::reminder::Reminder,
    ) -> anyhow::Result<i64>;

    /// Removes a reminder from a user's reminder list.
    ///
    /// # Arguments
    /// * `reminder_id` - The unique identifier of the reminder to delete
    /// * `user_id` - The user's unique identifier (for authorization)
    async fn delete_user_reminder(&self, reminder_id: i64, user_id: i64) -> anyhow::Result<()>;
}

/// Type alias for a boxed implementation of the AppRepo trait.
///
/// This allows for dynamic dispatch and easy dependency injection
/// throughout the application.
pub type ImplAppRepo = Box<dyn AppRepo>;
