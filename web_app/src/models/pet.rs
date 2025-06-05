use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Default, Clone)]
pub struct Pet {
    pub id: i64,
    pub external_id: Uuid,
    pub user_app_id: i64,
    pub pet_name: String,
    pub birthday: NaiveDate,
    pub breed: String,
    pub about: String,
    pub is_female: bool,
    pub is_lost: bool,
    pub is_spaying_neutering: bool,
    pub last_weight: Option<f64>,
    pub pic: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Display, Clone, Default, Deserialize, Serialize, PartialEq)]
pub enum PetHealthType {
    #[display("vaccine")]
    #[serde(alias = "vaccine", rename(serialize = "vaccine"))]
    Vaccine,
    #[default]
    #[display("deworm")]
    #[serde(alias = "deworm", rename(serialize = "deworm"))]
    Deworm,
    #[display("weight")]
    #[serde(alias = "weight", rename(serialize = "weight"))]
    Weight,
}

pub struct ExternalIdMetadata {
    pub external_id: Uuid,
    pub is_linked: bool,
}

#[derive(sqlx::FromRow)]
pub struct PetWeight {
    pub id: i64,
    pub pet_id: i64,
    pub value: f64,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct PetHealth {
    pub id: i64,
    pub pet_id: i64,
    pub health_record: PetHealthType,
    pub description: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, sqlx::FromRow)]
pub struct PetNote {
    pub id: i64,
    pub pet_id: i64,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
