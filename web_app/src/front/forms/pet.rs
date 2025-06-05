use crate::{front::utils, models};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct PetPic {
    pub body: Vec<u8>,
    pub filename_extension: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct CreatePetForm {
    pub id: i64,
    pub pet_full_name: String,
    pub pet_birthday: NaiveDate,
    pub pet_breed: String,
    pub is_lost: bool,
    pub is_spaying_neutering: bool,
    pub is_female: bool,
    pub about_pet: String,
    pub pet_pic: Option<PetPic>,
    pub pet_external_id: Option<Uuid>,
}

impl From<CreatePetForm> for models::pet::Pet {
    fn from(val: CreatePetForm) -> Self {
        let now = Utc::now();
        models::pet::Pet {
            id: val.id,
            pet_name: val.pet_full_name,
            birthday: val.pet_birthday,
            breed: val.pet_breed,
            about: val.about_pet,
            is_female: val.is_female,
            is_lost: val.is_lost,
            is_spaying_neutering: val.is_spaying_neutering,
            external_id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
            ..models::pet::Pet::default()
        }
    }
}

impl CreatePetForm {
    pub fn get_pet_pic_filename(&self) -> String {
        utils::filter_only_alphanumeric_chars(&self.pet_full_name).to_lowercase()
    }

    pub fn get_pic_storage_path(&self, user_email: &str) -> Option<String> {
        self.pet_pic.as_ref().map(|pic| {
            let pic_filename = self.get_pet_pic_filename();
            let pic_extension = pic.filename_extension.to_string();

            format!("pics/{user_email}/{pic_filename}.{pic_extension}").to_lowercase()
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PetNoteForm {
    pub title: String,
    pub body: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct HealthRecordForm {
    pub value: String,
    pub date: chrono::NaiveDate,
}
