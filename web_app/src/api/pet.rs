//! # Pet API Module
//!
//! This module contains all pet-related business logic including pet management,
//! health records, profiles, and public information handling. It serves as the
//! core domain logic for pet operations in the application.

use crate::{front, models, repo, services};
use anyhow::bail;
use chrono::{NaiveDate, NaiveDateTime, Utc};
use derive_more::Display;
use serde::Serialize;
use std::path::Path;
use uuid::Uuid;

/// Updates an existing pet or creates a new one based on the insert flag.
///
/// This internal function handles both pet creation and updates with a unified
/// interface. It validates external IDs, processes pet information, and manages
/// file uploads for pet pictures.
///
/// # Arguments
/// * `user_id` - ID of the user who owns the pet
/// * `user_email` - Email of the user (used for file storage paths)
/// * `insert` - True for creation, false for update
/// * `pet_info` - Pet form data including all pet details
/// * `repo` - Repository instance for database operations
/// * `storage_service` - Service for handling file uploads
///
/// # Returns
/// * `anyhow::Result<()>` - Success confirmation or error details
///
/// # Process
/// 1. Validate external ID if provided (for creation only)
/// 2. Convert form data to pet model
/// 3. Insert or update pet in database
/// 4. Handle pet picture upload if provided
/// 5. Update user subscription status if creating
///
/// # Errors
/// Returns an error if:
/// - External ID validation fails
/// - Database operations fail
/// - File upload fails
async fn update_or_create_pet(
    user_id: i64,
    user_email: &str,
    insert: bool,
    pet_info: front::forms::pet::CreatePetForm,
    repo: &repo::ImplAppRepo,
    storage_service: &services::ImplStorageService,
) -> anyhow::Result<()> {
    let _span = logfire::span!("update_or_create_pet").entered();

    if let Some(external_id) = pet_info.pet_external_id {
        let is_external_id_valid = repo
            .is_pet_external_id_linked(&external_id)
            .await?
            .map(|is_linked| !is_linked)
            .unwrap_or_else(|| false)
            && insert;

        if !is_external_id_valid {
            bail!("invalid pet_external_id")
        }
    }

    let pic_filename = pet_info.get_pet_pic_filename();
    let pet: models::pet::Pet = models::pet::Pet {
        user_app_id: user_id,
        pic: pet_info.get_pic_storage_path(user_email),
        external_id: pet_info.pet_external_id.unwrap_or_else(Uuid::new_v4),
        ..pet_info.clone().into()
    };

    if insert {
        repo.save_pet(&pet).await?;

        repo.set_user_as_subscribed(user_id).await?;
    } else {
        // update
        repo.update_pet(&pet).await?;
    }

    if let Some(pet_pic) = pet_info.pet_pic {
        storage_service
            .save_pic(user_email, &pic_filename, pet_pic.body)
            .await?;
    }

    Ok(())
}

/// State information required for adding a new pet to a user.
///
/// Contains user context and balance information needed to process
/// new pet creation, including balance verification and deduction.
pub struct UserStateAddNewPet {
    /// ID of the user adding the pet
    pub user_id: i64,
    /// Email address of the user (used for file storage paths)
    pub user_email: String,
    /// Current pet balance available for creating new pets
    pub pet_balance: u32,
}

/// Adds a new pet to a user's account and decrements their pet balance.
///
/// This is the main flow for pet creation that handles both the pet creation
/// process and balance management. It ensures the user has sufficient balance
/// and decrements it by 1 upon successful pet creation.
///
/// # Arguments
/// * `user_state` - User context and balance information
/// * `pet_info` - Pet form data with all pet details
/// * `repo` - Repository instance for database operations
/// * `storage_service` - Service for handling file uploads
///
/// # Returns
/// * `anyhow::Result<()>` - Success confirmation or error details
///
/// # Process
/// 1. Create the pet using the internal update_or_create_pet function
/// 2. Decrement user's pet balance by 1 if balance is available
///
/// # Errors
/// Returns an error if:
/// - Pet creation fails (validation, database, file upload)
/// - Balance update fails
pub async fn add_new_pet_to_user(
    user_state: UserStateAddNewPet,
    pet_info: front::forms::pet::CreatePetForm,
    repo: &repo::ImplAppRepo,
    storage_service: &services::ImplStorageService,
) -> anyhow::Result<()> {
    update_or_create_pet(
        user_state.user_id,
        &user_state.user_email,
        true,
        pet_info,
        repo,
        storage_service,
    )
    .await?;

    if user_state.pet_balance > 0 {
        repo.set_pet_balance(user_state.user_id, user_state.pet_balance - 1)
            .await?;
    }

    Ok(())
}

/// Updates an existing pet's information.
///
/// Modifies pet details without affecting the user's pet balance.
/// This function handles pet updates including picture changes.
///
/// # Arguments
/// * `user_id` - ID of the user who owns the pet
/// * `user_email` - Email of the user (used for file storage paths)
/// * `pet_info` - Updated pet form data
/// * `repo` - Repository instance for database operations
/// * `storage_service` - Service for handling file uploads
///
/// # Returns
/// * `anyhow::Result<()>` - Success confirmation or error details
pub async fn update_pet_to_user(
    user_id: i64,
    user_email: &str,
    pet_info: front::forms::pet::CreatePetForm,
    repo: &repo::ImplAppRepo,
    storage_service: &services::ImplStorageService,
) -> anyhow::Result<()> {
    update_or_create_pet(user_id, user_email, false, pet_info, repo, storage_service).await
}

/// Pet sex/gender enumeration.
///
/// Represents the biological sex of a pet with appropriate serialization
/// for Spanish language display and male/female display formats.
#[derive(Debug, Display, Default, Serialize)]
pub enum Sex {
    #[serde(rename(serialize = "macho"))]
    #[display("male")]
    Male,
    #[serde(rename(serialize = "hembra"))]
    #[default]
    #[display("female")]
    Female,
}

/// Schema for displaying pets in a list format.
///
/// Contains essential pet information optimized for list views,
/// including formatted age and basic identification details.
#[derive(Debug, Serialize)]
pub struct PetListSchema {
    /// Internal pet database ID
    pub id: i64,
    /// Public UUID for external references
    pub external_id: Uuid,
    /// Pet's name
    pub name: String,
    /// Pet's breed
    pub breed: String,
    /// Pet's biological sex
    pub sex: Sex,
    /// Human-readable formatted age string
    pub fmt_age: String,
}

/// Converts a Pet model to PetListSchema for display.
///
/// Transforms database pet data into a format suitable for list views,
/// including age calculation and sex conversion.
impl From<models::pet::Pet> for PetListSchema {
    fn from(val: models::pet::Pet) -> Self {
        PetListSchema {
            id: val.id,
            external_id: val.external_id,
            name: val.pet_name,
            breed: val.breed,
            sex: match val.is_female {
                true => Sex::Female,
                false => Sex::Male,
            },
            fmt_age: front::utils::fmt_dates_difference(
                val.birthday,
                front::utils::get_utc_now_with_default_time().date_naive(),
            ),
        }
    }
}

/// Retrieves all pets belonging to a user in list format.
///
/// Gets all pets owned by the specified user and converts them
/// to PetListSchema format for display in pet lists.
///
/// # Arguments
/// * `user_id` - ID of the user to get pets for
/// * `repo` - Repository instance for database operations
///
/// # Returns
/// * `anyhow::Result<Vec<PetListSchema>>` - List of pets in display format
pub async fn get_user_pets_cards(
    user_id: i64,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<Vec<PetListSchema>> {
    Ok(repo
        .get_all_pets_user_id(user_id)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

/// Retrieves pet data formatted for editing forms.
///
/// Gets a specific pet owned by the user and converts it to
/// form format suitable for editing interfaces.
///
/// # Arguments
/// * `pet_id` - ID of the pet to retrieve
/// * `user_id` - ID of the user who owns the pet
/// * `repo` - Repository instance for database operations
///
/// # Returns
/// * `anyhow::Result<front::forms::pet::CreatePetForm>` - Pet data in form format
pub async fn get_pet_user_to_edit(
    pet_id: i64,
    user_id: i64,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<front::forms::pet::CreatePetForm> {
    let pet = repo.get_pet_by_id(pet_id, user_id).await?;

    Ok(front::forms::pet::CreatePetForm {
        id: pet.id,
        pet_full_name: pet.pet_name,
        pet_birthday: pet.birthday,
        pet_breed: pet.breed,
        is_lost: pet.is_lost,
        is_spaying_neutering: pet.is_spaying_neutering,
        is_female: pet.is_female,
        about_pet: pet.about,
        pet_pic: pet.pic.map(|path| front::forms::pet::PetPic {
            body: vec![],
            filename_extension: path,
        }),
        pet_external_id: Some(pet.external_id),
    })
}

/// Schema for displaying pet information on public profiles.
///
/// Contains comprehensive pet information suitable for public viewing,
/// including health status and contact-enabling details.
#[derive(Debug, Serialize)]
pub struct PetPublicInfoSchema {
    /// Public UUID as string for external references
    pub external_id: String,
    /// Pet's name
    pub name: String,
    /// Pet's biological sex
    pub sex: Sex,
    /// Pet's breed information
    pub pet_breed: String,
    /// Most recent weight record if available
    pub last_weight: Option<f64>,
    /// Human-readable formatted age string
    pub fmt_age: String,
    /// Whether the pet is spayed/neutered
    pub is_spaying_neutering: bool,
    /// Whether the pet is currently lost
    pub is_lost: bool,
    /// Description and additional information about the pet
    pub about_pet: String,
    /// Whether the pet has a picture available
    pub pic: Option<String>,
}

/// Converts a Pet model to PetPublicInfoSchema for public display.
///
/// Transforms database pet data into a format suitable for public
/// viewing, including all relevant information for found pet scenarios.
impl From<models::pet::Pet> for PetPublicInfoSchema {
    fn from(val: models::pet::Pet) -> Self {
        PetPublicInfoSchema {
            external_id: val.external_id.to_string(),
            name: val.pet_name,
            sex: match val.is_female {
                true => Sex::Female,
                false => Sex::Male,
            },
            pic: val.pic,
            pet_breed: val.breed,
            last_weight: val.last_weight,
            fmt_age: front::utils::fmt_dates_difference(
                val.birthday,
                front::utils::get_utc_now_with_default_time().date_naive(),
            ),
            is_spaying_neutering: val.is_spaying_neutering,
            is_lost: val.is_lost,
            about_pet: val.about,
        }
    }
}

/// Retrieves public pet information by external ID.
///
/// Gets pet data using the public external ID and formats it
/// for public display. Used for QR code scanning and public access.
///
/// # Arguments
/// * `pet_external_id` - Public UUID of the pet
/// * `repo` - Repository instance for database operations
///
/// # Returns
/// * `anyhow::Result<PetPublicInfoSchema>` - Pet information for public display
pub async fn get_pet_public_info(
    pet_external_id: Uuid,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<PetPublicInfoSchema> {
    let pet = repo.get_pet_by_external_id(pet_external_id).await?;
    Ok(pet.into())
}

/// Retrieves metadata about a pet's external ID.
///
/// Checks if an external ID exists and whether it's linked to a pet.
/// Used for validation during pet creation and external ID verification.
///
/// # Arguments
/// * `pet_external_id` - External UUID to check
/// * `repo` - Repository instance for database operations
///
/// # Returns
/// * `anyhow::Result<Option<models::pet::ExternalIdMetadata>>` - Metadata if ID exists
pub async fn get_pet_external_id_metadata(
    pet_external_id: &Uuid,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<Option<models::pet::ExternalIdMetadata>> {
    if let Some(is_linked) = repo.is_pet_external_id_linked(pet_external_id).await? {
        return Ok(Some(models::pet::ExternalIdMetadata {
            external_id: *pet_external_id,
            is_linked,
        }));
    }

    Ok(None)
}

/// Structure for pet picture data with file extension.
///
/// Contains the raw image bytes and file extension information
/// for serving pet pictures in public contexts.
#[derive(Debug, Serialize, Default)]
pub struct PetPublicPic {
    /// Raw image file bytes
    pub body: Vec<u8>,
    /// File extension (jpg, png, etc.)
    pub extension: String,
}

/// Retrieves a pet's picture for public display.
///
/// Gets the pet's picture file from storage using the external ID.
/// Returns the image bytes and file extension for serving to clients.
///
/// # Arguments
/// * `pet_external_id` - Public UUID of the pet
/// * `repo` - Repository instance for database operations
/// * `storage_service` - Service for file retrieval
///
/// # Returns
/// * `anyhow::Result<Option<PetPublicPic>>` - Picture data if available
pub async fn get_public_pic(
    pet_external_id: Uuid,
    repo: &repo::ImplAppRepo,
    storage_service: &services::ImplStorageService,
) -> anyhow::Result<Option<PetPublicPic>> {
    if let Some(pic_path) = repo
        .get_pet_pic_path_by_external_id(pet_external_id)
        .await?
    {
        let pic_path = Path::new(&pic_path);

        return Ok(Some(PetPublicPic {
            body: storage_service
                .get_pic_as_bytes(pic_path.with_extension("").to_str().unwrap_or_default())
                .await?,
            extension: pic_path
                .extension()
                .and_then(|p| p.to_str())
                .unwrap_or("png")
                .to_string(),
        }));
    }

    Ok(None)
}

/// Unified structure for pet health records.
///
/// Represents health records (weight, vaccines, deworms) in a consistent
/// format for display and API responses.
#[derive(Default, serde::Serialize, Debug)]
pub struct PetHealthRecord {
    /// Record ID for deletion and updates
    pub id: i64,
    /// Record value (weight amount, vaccine name, etc.)
    pub value: String,
    /// Date when the record was created
    pub date: NaiveDateTime,
}

/// Converts a PetWeight model to PetHealthRecord.
///
/// Formats weight values to 2 decimal places for consistent display.
impl From<models::pet::PetWeight> for PetHealthRecord {
    fn from(val: models::pet::PetWeight) -> Self {
        PetHealthRecord {
            id: val.id,
            value: format!("{:.2}", val.value),
            date: val.created_at,
        }
    }
}

/// Converts a PetHealth model to PetHealthRecord.
///
/// Used for vaccine and deworming records.
impl From<models::pet::PetHealth> for PetHealthRecord {
    fn from(val: models::pet::PetHealth) -> Self {
        PetHealthRecord {
            id: val.id,
            value: val.description,
            date: val.created_at,
        }
    }
}

/// Retrieves health records for a pet by type.
///
/// Gets specific types of health records (weight, vaccine, deworm)
/// for a pet, with optional user ownership verification.
///
/// # Arguments
/// * `pet_external_id` - Public UUID of the pet
/// * `health_record` - Type of health record to retrieve
/// * `user_id` - Optional user ID for ownership verification
/// * `repo` - Repository instance for database operations
///
/// # Returns
/// * `anyhow::Result<Vec<PetHealthRecord>>` - List of health records
pub async fn get_pet_health_records(
    pet_external_id: Uuid,
    health_record: &models::pet::PetHealthType,
    user_id: Option<i64>,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<Vec<PetHealthRecord>> {
    match health_record {
        models::pet::PetHealthType::Weight => Ok(repo
            .get_pet_weights(pet_external_id, user_id)
            .await?
            .into_iter()
            .map(Into::into)
            .collect()),
        _ => Ok(repo
            .get_pet_health_records(pet_external_id, user_id, health_record.clone())
            .await?
            .into_iter()
            .map(Into::into)
            .collect()),
    }
}

/// Deletes a pet and all associated information.
///
/// Removes the pet and all related data (health records, notes, etc.)
/// from the database. This operation is irreversible.
///
/// # Arguments
/// * `pet_id` - ID of the pet to delete
/// * `user_id` - ID of the user who owns the pet
/// * `repo` - Repository instance for database operations
///
/// # Returns
/// * `anyhow::Result<()>` - Success confirmation or error details
pub async fn delete_pet_and_its_info(
    pet_id: i64,
    user_id: i64,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<()> {
    repo.delete_pet(pet_id, user_id).await?;
    Ok(())
}

/// Adds a new health record to a pet.
///
/// Creates a new health record (weight, vaccine, or deworm) for the specified
/// pet with the provided information and date.
///
/// # Arguments
/// * `pet_external_id` - Public UUID of the pet
/// * `health_record` - Type of health record to create
/// * `user_id` - ID of the user who owns the pet
/// * `desc` - Record description/value (weight amount, vaccine name, etc.)
/// * `date` - Date for the health record
/// * `repo` - Repository instance for database operations
///
/// # Returns
/// * `anyhow::Result<PetHealthRecord>` - The created health record
pub async fn insert_pet_health_record(
    pet_external_id: Uuid,
    health_record: &models::pet::PetHealthType,
    user_id: i64,
    desc: String,
    date: NaiveDate,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<PetHealthRecord> {
    match health_record {
        models::pet::PetHealthType::Weight => Ok(repo
            .insert_pet_weight(
                pet_external_id,
                user_id,
                desc.parse::<f64>().unwrap_or(0.0),
                date,
            )
            .await?
            .into()),
        models::pet::PetHealthType::Vaccine => Ok(repo
            .insert_vaccine_to(pet_external_id, user_id, desc, date)
            .await?
            .into()),
        models::pet::PetHealthType::Deworm => Ok(repo
            .insert_deworm_to(pet_external_id, user_id, desc, date)
            .await?
            .into()),
    }
}

/// Deletes a specific health record from a pet.
///
/// Removes a health record (weight, vaccine, or deworm) from the pet's
/// health history. Requires ownership verification.
///
/// # Arguments
/// * `record_id` - ID of the health record to delete
/// * `pet_external_id` - Public UUID of the pet
/// * `user_id` - ID of the user who owns the pet
/// * `health_record` - Type of health record being deleted
/// * `repo` - Repository instance for database operations
///
/// # Returns
/// * `anyhow::Result<()>` - Success confirmation or error details
pub async fn delete_pet_health_record(
    record_id: i64,
    pet_external_id: Uuid,
    user_id: i64,
    health_record: &models::pet::PetHealthType,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<()> {
    match health_record {
        models::pet::PetHealthType::Weight => Ok(repo
            .delete_pet_weight(pet_external_id, user_id, record_id)
            .await?),
        models::pet::PetHealthType::Vaccine => Ok(repo
            .delete_vaccine(pet_external_id, user_id, record_id)
            .await?),
        models::pet::PetHealthType::Deworm => Ok(repo
            .delete_deworm(pet_external_id, user_id, record_id)
            .await?),
    }
}

/// Retrieves all notes for a specific pet.
///
/// Gets all user-created notes associated with the pet.
/// Notes are private to the pet owner.
///
/// # Arguments
/// * `user_id` - ID of the user who owns the pet
/// * `pet_id` - ID of the pet to get notes for
/// * `repo` - Repository instance for database operations
///
/// # Returns
/// * `anyhow::Result<Vec<models::pet::PetNote>>` - List of pet notes
pub async fn get_pet_notes(
    user_id: i64,
    pet_id: i64,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<Vec<models::pet::PetNote>> {
    repo.get_pet_notes(user_id, pet_id).await
}

/// Information for creating a new pet note.
///
/// Contains the title and content for a new note to be added to a pet.
pub struct PetNoteInfo {
    /// Title of the note
    pub title: String,
    /// Main content/body of the note
    pub body: String,
}

/// Converts a PetNoteForm to PetNoteInfo.
///
/// Transforms form data into the structure needed for note creation.
impl From<front::forms::pet::PetNoteForm> for PetNoteInfo {
    fn from(val: front::forms::pet::PetNoteForm) -> Self {
        PetNoteInfo {
            title: val.title,
            body: val.body,
        }
    }
}

/// Adds a new note to a pet.
///
/// Creates a new note with the provided title and content,
/// associating it with the specified pet and user.
///
/// # Arguments
/// * `user_id` - ID of the user creating the note
/// * `pet_id` - ID of the pet to add the note to
/// * `note_info` - Note title and content
/// * `repo` - Repository instance for database operations
///
/// # Returns
/// * `anyhow::Result<()>` - Success confirmation or error details
pub async fn add_new_note(
    user_id: i64,
    pet_id: i64,
    note_info: PetNoteInfo,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<()> {
    repo.insert_new_pet_note(
        user_id,
        &models::pet::PetNote {
            id: 0,
            pet_id,
            title: note_info.title.to_string(),
            content: note_info.body.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
    )
    .await?;
    Ok(())
}

/// Deletes a specific note from a pet.
///
/// Removes a note from the pet's note collection. Requires ownership
/// verification to ensure only the pet owner can delete notes.
///
/// # Arguments
/// * `note_id` - ID of the note to delete
/// * `pet_id` - ID of the pet the note belongs to
/// * `user_id` - ID of the user who owns the pet
/// * `repo` - Repository instance for database operations
///
/// # Returns
/// * `anyhow::Result<()>` - Success confirmation or error details
pub async fn delete_note(
    note_id: i64,
    pet_id: i64,
    user_id: i64,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<()> {
    repo.delete_pet_note(pet_id, user_id, note_id).await?;
    Ok(())
}

/// Complete pet information including all related data.
///
/// Aggregates all pet-related information including the pet details,
/// health records, and notes for comprehensive display or export.
pub struct PetFullInfo {
    /// Core pet information
    pub pet: models::pet::Pet,
    /// All vaccine records for the pet
    pub vaccines: Vec<models::pet::PetHealth>,
    /// All deworming records for the pet
    pub deworms: Vec<models::pet::PetHealth>,
    /// All weight records for the pet
    pub weights: Vec<models::pet::PetWeight>,
    /// All notes associated with the pet
    pub notes: Vec<models::pet::PetNote>,
}

/// Retrieves complete pet information with all related data.
///
/// Gets the pet details along with all health records, notes, and other
/// associated information for comprehensive display or data export.
///
/// # Arguments
/// * `pet_id` - ID of the pet to get information for
/// * `user_id` - ID of the user who owns the pet
/// * `repo` - Repository instance for database operations
///
/// # Returns
/// * `anyhow::Result<PetFullInfo>` - Complete pet information structure
pub async fn get_full_info(
    pet_id: i64,
    user_id: i64,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<PetFullInfo> {
    let pet = repo.get_pet_by_id(pet_id, user_id).await?;
    let external_id = pet.external_id;
    let pet_id = pet.id;

    Ok(PetFullInfo {
        pet,
        vaccines: repo
            .get_pet_health_records(
                external_id,
                Some(user_id),
                models::pet::PetHealthType::Vaccine,
            )
            .await?,
        deworms: repo
            .get_pet_health_records(
                external_id,
                Some(user_id),
                models::pet::PetHealthType::Deworm,
            )
            .await?,
        weights: repo.get_pet_weights(external_id, Some(user_id)).await?,
        notes: repo.get_pet_notes(user_id, pet_id).await?,
    })
}
