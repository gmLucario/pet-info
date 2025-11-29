use crate::models;
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct CropperBox {
    pub x: u32,
    pub y: u32,
    pub diameter: u32,
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
    pub pet_pic: Option<crate::models::Pic>,
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
    pub fn build_pic_storage_path(&self, external_id: Uuid) -> Option<String> {
        self.pet_pic.as_ref().map(|_| format!("pics/{external_id}"))
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
