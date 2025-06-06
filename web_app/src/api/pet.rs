use crate::{front, models, repo, services};
use anyhow::bail;
use chrono::{NaiveDate, NaiveDateTime, Utc};
use derive_more::Display;
use serde::Serialize;
use std::path::Path;
use uuid::Uuid;

async fn update_or_create_pet(
    user_id: i64,
    user_email: &str,
    insert: bool,
    pet_info: front::forms::pet::CreatePetForm,
    repo: &repo::ImplAppRepo,
    storage_service: &services::ImplStorageService,
) -> anyhow::Result<()> {
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

pub struct UserStateAddNewPet {
    pub user_id: i64,
    pub user_email: String,
    pub pet_balance: u32,
}

/// Flow to add a new pet to a user
/// Calling this function will reduce the user pet balance 1 unit value
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

pub async fn update_pet_to_user(
    user_id: i64,
    user_email: &str,
    pet_info: front::forms::pet::CreatePetForm,
    repo: &repo::ImplAppRepo,
    storage_service: &services::ImplStorageService,
) -> anyhow::Result<()> {
    update_or_create_pet(user_id, user_email, false, pet_info, repo, storage_service).await
}

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

#[derive(Debug, Serialize)]
pub struct PetListSchema {
    pub id: i64,
    pub external_id: Uuid,
    pub name: String,
    pub breed: String,
    pub sex: Sex,
    pub fmt_age: String,
}

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

#[derive(Debug, Serialize)]
pub struct PetPublicInfoSchema {
    pub external_id: String,
    pub name: String,
    pub sex: Sex,
    pub pet_breed: String,
    pub last_weight: Option<f64>,
    pub fmt_age: String,
    pub is_spaying_neutering: bool,
    pub is_lost: bool,
    pub about_pet: String,
}

impl From<models::pet::Pet> for PetPublicInfoSchema {
    fn from(val: models::pet::Pet) -> Self {
        PetPublicInfoSchema {
            external_id: val.external_id.to_string(),
            name: val.pet_name,
            sex: match val.is_female {
                true => Sex::Female,
                false => Sex::Male,
            },
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

pub async fn get_pet_public_info(
    pet_external_id: Uuid,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<PetPublicInfoSchema> {
    let pet = repo.get_pet_by_external_id(pet_external_id).await?;
    Ok(pet.into())
}

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

#[derive(Debug, Serialize, Default)]
pub struct PetPublicPic {
    pub body: Vec<u8>,
    pub extension: String,
}

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
                .unwrap_or("jpg")
                .to_string(),
        }));
    }

    Ok(None)
}

#[derive(Default, serde::Serialize, Debug)]
pub struct PetHealthRecord {
    pub id: i64,
    pub value: String,
    pub date: NaiveDateTime,
}

impl From<models::pet::PetWeight> for PetHealthRecord {
    fn from(val: models::pet::PetWeight) -> Self {
        PetHealthRecord {
            id: val.id,
            value: format!("{:.2}", val.value),
            date: val.created_at,
        }
    }
}

impl From<models::pet::PetHealth> for PetHealthRecord {
    fn from(val: models::pet::PetHealth) -> Self {
        PetHealthRecord {
            id: val.id,
            value: val.description,
            date: val.created_at,
        }
    }
}

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

pub async fn delete_pet_and_its_info(
    pet_id: i64,
    user_id: i64,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<()> {
    repo.delete_pet(pet_id, user_id).await?;
    Ok(())
}

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

pub async fn get_pet_notes(
    user_id: i64,
    pet_id: i64,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<Vec<models::pet::PetNote>> {
    repo.get_pet_notes(user_id, pet_id).await
}

pub struct PetNoteInfo {
    pub title: String,
    pub body: String,
}

impl From<front::forms::pet::PetNoteForm> for PetNoteInfo {
    fn from(val: front::forms::pet::PetNoteForm) -> Self {
        PetNoteInfo {
            title: val.title,
            body: val.body,
        }
    }
}

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

pub async fn delete_note(
    note_id: i64,
    pet_id: i64,
    user_id: i64,
    repo: &repo::ImplAppRepo,
) -> anyhow::Result<()> {
    repo.delete_pet_note(pet_id, user_id, note_id).await?;
    Ok(())
}

pub struct PetFullInfo {
    pub pet: models::pet::Pet,
    pub vaccines: Vec<models::pet::PetHealth>,
    pub deworms: Vec<models::pet::PetHealth>,
    pub weights: Vec<models::pet::PetWeight>,
    pub notes: Vec<models::pet::PetNote>,
}

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
