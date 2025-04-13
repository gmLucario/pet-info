use crate::models;
use async_trait::async_trait;
use chrono::Utc;
use serde_json::from_str;
use sqlx::{FromRow, Row, SqlitePool, sqlite::SqliteRow};
use uuid::Uuid;

use super::{AppRepo, sqlite_queries};

#[derive(Clone)]
pub struct SqlxSqliteRepo {
    pub db_pool: SqlitePool,
}

impl FromRow<'_, SqliteRow> for models::pet::Pet {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let external_id: uuid::fmt::Hyphenated = row.try_get("external_id")?;

        Ok(Self {
            id: row.try_get("id")?,
            external_id: external_id.into(),
            user_app_id: row.try_get("user_app_id")?,
            pet_name: row.try_get("pet_name")?,
            birthday: row.try_get("birthday")?,
            breed: row.try_get("breed")?,
            about: row.try_get("about")?,
            is_female: row.try_get("is_female")?,
            is_lost: row.try_get("is_lost")?,
            is_spaying_neutering: row.try_get("is_spaying_neutering")?,
            last_weight: row.try_get("last_weight")?,
            pic: row.try_get("pic")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[async_trait]
impl AppRepo for SqlxSqliteRepo {
    async fn remove_user_app_data(&self, user_id: i64) -> anyhow::Result<()> {
        Ok(sqlx::query(sqlite_queries::QUERY_DELETE_USER_APP_DATA)
            .bind(user_id)
            .bind(Utc::now())
            .execute(&self.db_pool)
            .await
            .map(|_| ())?)
    }

    async fn set_user_as_active(&self, user_id: i64) -> anyhow::Result<()> {
        Ok(
            sqlx::query("UPDATE user_app SET is_enabled=1,updated_at=$2 WHERE id = $1;")
                .bind(user_id)
                .bind(Utc::now())
                .execute(&self.db_pool)
                .await
                .map(|_| ())?,
        )
    }

    async fn get_user_app_by_email(
        &self,
        email: &str,
    ) -> anyhow::Result<Option<models::user_app::User>> {
        let user = sqlx::query(sqlite_queries::QUERY_GET_USER_APP_BY_EMAIL)
            .bind(email)
            .map(|row: sqlx::sqlite::SqliteRow| models::user_app::User {
                id: row.try_get("id").unwrap_or(-1),
                email: row.try_get("email").unwrap_or_default(),
                phone_reminder: row.try_get("phone_reminder").unwrap_or_default(),
                account_role: serde_json::from_str::<models::user_app::AccountRole>(&format!(
                    "\"{}\"",
                    row.try_get::<String, &str>("account_role")
                        .unwrap_or_default()
                ))
                .unwrap_or_default(),
                is_subscribed: row.try_get("is_subscribed").unwrap_or_default(),
                is_enabled: row.try_get("is_enabled").unwrap_or(true),
                created_at: row.try_get("created_at").unwrap_or_default(),
                updated_at: row.try_get("updated_at").unwrap_or_default(),
            })
            .fetch_optional(&self.db_pool)
            .await?;

        Ok(user)
    }

    async fn insert_verified_phone_to_user_app(
        &self,
        user_app_id: i64,
        phone: &str,
    ) -> anyhow::Result<()> {
        sqlx::query("UPDATE user_app SET phone_reminder=$1, updated_at=$2 WHERE id=$3;")
            .bind(phone)
            .bind(Utc::now())
            .bind(user_app_id)
            .execute(&self.db_pool)
            .await?;
        Ok(())
    }

    async fn set_to_null_verified_phone(&self, user_app_id: i64) -> anyhow::Result<()> {
        sqlx::query("UPDATE user_app SET phone_reminder=NULL, updated_at=$1 WHERE id=$2;")
            .bind(Utc::now())
            .bind(user_app_id)
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }

    async fn save_user_app(&self, app_user: &models::user_app::User) -> anyhow::Result<i64> {
        let user_app_id = sqlx::query(
            "INSERT INTO user_app(email,account_role,created_at,updated_at) VALUES($1,$2,$3,$4);",
        )
        .bind(&app_user.email)
        .bind(app_user.account_role.to_string())
        .bind(app_user.created_at)
        .bind(app_user.updated_at)
        .execute(&self.db_pool)
        .await?
        .last_insert_rowid();

        Ok(user_app_id)
    }

    async fn get_user_payments(
        &self,
        user_id: i64,
    ) -> anyhow::Result<Vec<models::payment::Payment>> {
        Ok(sqlx::query(sqlite_queries::QUERY_GET_USER_PAYM_SUBS)
            .bind(user_id)
            .map(|row: sqlx::sqlite::SqliteRow| models::payment::Payment {
                user_id: row.try_get("user_id").unwrap_or(-1),
                mp_paym_id: from_str::<usize>(
                    row.try_get::<&str, &str>("mp_paym_id").unwrap_or("0"),
                )
                .unwrap_or(0),
                payment_idempotency_h: row.try_get("payment_idempotency_h").unwrap_or_default(),
                transaction_amount: row.try_get("transaction_amount").unwrap_or("0.00".into()),
                installments: row.try_get("installments").unwrap_or(1),
                payment_method_id: row.try_get("payment_method_id").unwrap_or_default(),
                issuer_id: row.try_get("issuer_id").unwrap_or_default(),
                status: serde_json::from_str::<models::payment::PaymentStatus>(&format!(
                    "\"{}\"",
                    row.try_get::<String, &str>("status").unwrap_or_default()
                ))
                .unwrap_or_default(),
                created_at: row.try_get("created_at").unwrap_or_default(),
                updated_at: row.try_get("updated_at").unwrap_or_default(),
            })
            .fetch_all(&self.db_pool)
            .await?)
    }

    async fn save_subs_payment(&self, payment: &models::payment::Payment) -> anyhow::Result<i64> {
        let p_id = sqlx::query(sqlite_queries::QUERY_INSERT_NEW_SUB_PAYM)
            .bind(payment.user_id)
            .bind(payment.mp_paym_id.to_string())
            .bind(&payment.payment_idempotency_h)
            .bind(&payment.transaction_amount)
            .bind(payment.installments)
            .bind(&payment.payment_method_id)
            .bind(&payment.issuer_id)
            .bind(payment.status.to_string())
            .bind(payment.created_at)
            .bind(payment.updated_at)
            .execute(&self.db_pool)
            .await?
            .last_insert_rowid();

        Ok(p_id)
    }

    async fn set_user_as_subscribed(&self, user_id: i64) -> anyhow::Result<()> {
        sqlx::query("UPDATE user_app SET is_subscribed=1, updated_at=$1 WHERE id = $2;")
            .bind(Utc::now())
            .bind(user_id)
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }

    async fn save_or_update_pet(
        &self,
        pet: &models::pet::Pet,
        insert: bool,
    ) -> anyhow::Result<i64> {
        if insert {
            return Ok(sqlx::query(sqlite_queries::QUERY_INSERT_PET)
                .bind(pet.external_id.to_string())
                .bind(pet.user_app_id)
                .bind(&pet.pet_name)
                .bind(pet.birthday)
                .bind(&pet.breed)
                .bind(&pet.about)
                .bind(pet.is_female)
                .bind(pet.is_lost)
                .bind(pet.is_spaying_neutering)
                .bind(&pet.pic)
                .bind(pet.created_at)
                .bind(pet.updated_at)
                .execute(&self.db_pool)
                .await?
                .last_insert_rowid());
        }

        sqlx::query(sqlite_queries::QUERY_UPDATE_PET)
            .bind(pet.id)
            .bind(pet.user_app_id)
            .bind(&pet.pet_name)
            .bind(pet.birthday)
            .bind(&pet.breed)
            .bind(&pet.about)
            .bind(pet.is_female)
            .bind(pet.is_lost)
            .bind(pet.is_spaying_neutering)
            .bind(Utc::now())
            .execute(&self.db_pool)
            .await?;

        if let Some(pic_path) = &pet.pic {
            sqlx::query(
                "UPDATE pet SET pic=$3, updated_at = $4 WHERE id = $1 AND user_app_id = $2;",
            )
            .bind(pet.id)
            .bind(pet.user_app_id)
            .bind(pic_path)
            .bind(Utc::now())
            .execute(&self.db_pool)
            .await?;
        }

        Ok(pet.id)
    }

    async fn delete_pet(&self, pet_id: i64, user_id: i64) -> anyhow::Result<()> {
        sqlx::query(sqlite_queries::QUERY_DELETE_PET)
            .bind(pet_id)
            .bind(user_id)
            .execute(&self.db_pool)
            .await?;
        Ok(())
    }

    async fn get_all_pets_user_id(&self, user_id: i64) -> anyhow::Result<Vec<models::pet::Pet>> {
        Ok(
            sqlx::query_as::<_, models::pet::Pet>(sqlite_queries::QUERY_GET_ALL_PETS_USER_ID)
                .bind(user_id)
                .fetch_all(&self.db_pool)
                .await?,
        )
    }

    async fn get_pet_by_external_id(
        &self,
        pet_external_id: Uuid,
    ) -> anyhow::Result<models::pet::Pet> {
        Ok(
            sqlx::query_as::<_, models::pet::Pet>(sqlite_queries::QUERY_GET_PET_BY_EXTERNAL_ID)
                .bind(pet_external_id.to_string())
                .fetch_one(&self.db_pool)
                .await?,
        )
    }

    async fn get_pet_pic_path_by_external_id(
        &self,
        pet_external_id: Uuid,
    ) -> anyhow::Result<Option<String>> {
        let pet_pic_path: Option<String> =
            sqlx::query_scalar("SELECT p.pic FROM pet AS p WHERE p.external_id = $1;")
                .bind(pet_external_id.to_string())
                .fetch_optional(&self.db_pool)
                .await?;

        Ok(pet_pic_path)
    }

    async fn get_pet_by_id(&self, pet_id: i64, user_id: i64) -> anyhow::Result<models::pet::Pet> {
        Ok(
            sqlx::query_as::<_, models::pet::Pet>(sqlite_queries::QUERY_GET_PET_BY_ID)
                .bind(pet_id)
                .bind(user_id)
                .fetch_one(&self.db_pool)
                .await?,
        )
    }

    async fn get_pet_weights(
        &self,
        pet_external_id: Uuid,
        user_id: Option<i64>,
    ) -> anyhow::Result<Vec<models::pet::PetWeight>> {
        let pet_external_id = pet_external_id.to_string();
        let query = if let Some(user_id) = user_id {
            sqlx::query_as::<_, models::pet::PetWeight>(
                sqlite_queries::QUERY_GET_PET_WEIGHTS_BY_EXTERNAL_AND_USER_ID,
            )
            .bind(pet_external_id)
            .bind(user_id)
        } else {
            sqlx::query_as::<_, models::pet::PetWeight>(
                sqlite_queries::QUERY_GET_PET_WEIGHTS_BY_EXTERNAL_ID,
            )
            .bind(pet_external_id)
        };

        Ok(query.fetch_all(&self.db_pool).await?)
    }

    async fn get_pet_health_records(
        &self,
        pet_external_id: Uuid,
        user_id: Option<i64>,
        health_type: models::pet::PetHealthType,
    ) -> anyhow::Result<Vec<models::pet::PetHealth>> {
        let pet_external_id = pet_external_id.to_string();
        let query = if let Some(user_id) = user_id {
            sqlx::query(sqlite_queries::QUERY_GET_PET_HEALTH_RECORD)
                .bind(pet_external_id)
                .bind(user_id)
                .bind(health_type.to_string())
        } else {
            sqlx::query(sqlite_queries::QUERY_GET_PET_PUBLIC_HEALTH_RECORD)
                .bind(pet_external_id)
                .bind(health_type.to_string())
        };

        return Ok(query
            .map(|row: sqlx::sqlite::SqliteRow| models::pet::PetHealth {
                id: row.try_get("id").unwrap_or(-1),
                pet_id: row.try_get("pet_id").unwrap_or(-1),
                health_record: health_type.clone(),
                description: row.try_get("description").unwrap_or_default(),
                created_at: row.try_get("created_at").unwrap_or_default(),
            })
            .fetch_all(&self.db_pool)
            .await?);
    }

    async fn insert_vaccine_to(
        &self,
        pet_external_id: Uuid,
        user_id: i64,
        desc: String,
        date: chrono::NaiveDate,
    ) -> anyhow::Result<models::pet::PetHealth> {
        let date = date.and_time(chrono::NaiveTime::default());
        let health_record = models::pet::PetHealthType::Vaccine;

        let record = sqlx::query(sqlite_queries::QUERY_INSERT_PET_HEALTH_RECORD)
            .bind(pet_external_id.to_string())
            .bind(user_id)
            .bind(health_record.to_string())
            .bind(&desc)
            .bind(date)
            .map(|row: sqlx::sqlite::SqliteRow| models::pet::PetHealth {
                id: row.try_get("id").unwrap_or(-1),
                pet_id: row.try_get("pet_id").unwrap_or(-1),
                health_record: health_record.clone(),
                description: desc.to_string(),
                created_at: date,
            })
            .fetch_one(&self.db_pool)
            .await?;

        return Ok(record);
    }

    async fn insert_deworm_to(
        &self,
        pet_external_id: Uuid,
        user_id: i64,
        desc: String,
        date: chrono::NaiveDate,
    ) -> anyhow::Result<models::pet::PetHealth> {
        let health_record = models::pet::PetHealthType::Deworm;
        let date = date.and_time(chrono::NaiveTime::default());

        let record = sqlx::query(sqlite_queries::QUERY_INSERT_PET_HEALTH_RECORD)
            .bind(pet_external_id.to_string())
            .bind(user_id)
            .bind(health_record.to_string())
            .bind(&desc)
            .bind(date)
            .map(|row: sqlx::sqlite::SqliteRow| models::pet::PetHealth {
                id: row.try_get("id").unwrap_or(-1),
                pet_id: row.try_get("pet_id").unwrap_or(-1),
                health_record: health_record.clone(),
                description: desc.to_string(),
                created_at: date,
            })
            .fetch_one(&self.db_pool)
            .await?;

        return Ok(record);
    }

    async fn insert_pet_weight(
        &self,
        pet_external_id: Uuid,
        user_id: i64,
        weight: f64,
        date: chrono::NaiveDate,
    ) -> anyhow::Result<models::pet::PetWeight> {
        let date = date.and_time(chrono::NaiveTime::default());

        return Ok(sqlx::query_as::<_, models::pet::PetWeight>(
            sqlite_queries::QUERY_INSERT_PET_WEIGHT,
        )
        .bind(pet_external_id.to_string())
        .bind(user_id)
        .bind(weight)
        .bind(date)
        .fetch_one(&self.db_pool)
        .await?);
    }

    async fn delete_pet_weight(
        &self,
        pet_external_id: Uuid,
        user_id: i64,
        weight_id: i64,
    ) -> anyhow::Result<()> {
        sqlx::query(sqlite_queries::QUERY_DELETE_PET_WEIGHT)
            .bind(weight_id)
            .bind(pet_external_id.to_string())
            .bind(user_id)
            .execute(&self.db_pool)
            .await?;
        Ok(())
    }

    async fn delete_deworm(
        &self,
        pet_external_id: Uuid,
        user_id: i64,
        deworm_id: i64,
    ) -> anyhow::Result<()> {
        sqlx::query(sqlite_queries::QUERY_DELETE_PET_HEALTH_RECORD)
            .bind(deworm_id)
            .bind(models::pet::PetHealthType::Deworm.to_string())
            .bind(pet_external_id.to_string())
            .bind(user_id)
            .execute(&self.db_pool)
            .await?;
        Ok(())
    }

    async fn delete_vaccine(
        &self,
        pet_external_id: Uuid,
        user_id: i64,
        vaccine_id: i64,
    ) -> anyhow::Result<()> {
        sqlx::query(sqlite_queries::QUERY_DELETE_PET_HEALTH_RECORD)
            .bind(vaccine_id)
            .bind(models::pet::PetHealthType::Vaccine.to_string())
            .bind(pet_external_id.to_string())
            .bind(user_id)
            .execute(&self.db_pool)
            .await?;
        Ok(())
    }

    async fn get_pet_owner_contacts(
        &self,
        pet_external_id: Uuid,
    ) -> anyhow::Result<Vec<models::user_app::OwnerContact>> {
        return Ok(sqlx::query_as::<_, models::user_app::OwnerContact>(
            sqlite_queries::QUERY_GET_PET_OWNER_CONTACTS,
        )
        .bind(pet_external_id.to_string())
        .fetch_all(&self.db_pool)
        .await?);
    }

    async fn get_owner_contacts(
        &self,
        user_id: i64,
    ) -> anyhow::Result<Vec<models::user_app::OwnerContact>> {
        return Ok(sqlx::query_as::<_, models::user_app::OwnerContact>(
            sqlite_queries::QUERY_GET_OWNER_CONTACTS,
        )
        .bind(user_id)
        .fetch_all(&self.db_pool)
        .await?);
    }

    async fn insert_owner_contact(
        &self,
        user_id: i64,
        desc: String,
        contact: String,
    ) -> anyhow::Result<models::user_app::OwnerContact> {
        let now = Utc::now();
        let id = sqlx::query(sqlite_queries::QUERY_INSERT_NEW_OWNER_CONTACT)
            .bind(user_id)
            .bind(&desc)
            .bind(&contact)
            .bind(now)
            .execute(&self.db_pool)
            .await?
            .last_insert_rowid();
        Ok(models::user_app::OwnerContact {
            id,
            user_app_id: user_id,
            full_name: desc,
            contact_value: contact,
            created_at: now,
        })
    }

    async fn delete_owner_contact(&self, user_id: i64, contact_id: i64) -> anyhow::Result<()> {
        sqlx::query(sqlite_queries::QUERY_DELETE_OWNER_CONTACT)
            .bind(contact_id)
            .bind(user_id)
            .execute(&self.db_pool)
            .await?;
        Ok(())
    }

    async fn insert_new_pet_note(
        &self,
        user_id: i64,
        note: &models::pet::PetNote,
    ) -> anyhow::Result<i64> {
        Ok(sqlx::query(sqlite_queries::QUERY_INSERT_PET_NOTE)
            .bind(note.pet_id)
            .bind(user_id)
            .bind(&note.title)
            .bind(&note.content)
            .bind(note.created_at)
            .execute(&self.db_pool)
            .await?
            .last_insert_rowid())
    }

    async fn get_pet_notes(
        &self,
        user_id: i64,
        pet_id: i64,
    ) -> anyhow::Result<Vec<models::pet::PetNote>> {
        return Ok(
            sqlx::query_as::<_, models::pet::PetNote>(sqlite_queries::QUERY_GET_PET_NOTES)
                .bind(pet_id)
                .bind(user_id)
                .fetch_all(&self.db_pool)
                .await?,
        );
    }

    async fn delete_pet_note(&self, pet_id: i64, user_id: i64, note_id: i64) -> anyhow::Result<()> {
        sqlx::query(sqlite_queries::QUERY_DELETE_PET_NOTE)
            .bind(note_id)
            .bind(pet_id)
            .bind(user_id)
            .execute(&self.db_pool)
            .await?;
        Ok(())
    }

    async fn get_active_user_remiders(
        &self,
        user_id: i64,
    ) -> anyhow::Result<Vec<models::reminder::Reminder>> {
        return Ok(sqlx::query(sqlite_queries::QUERY_GET_USER_ACTIVE_REMINDERS)
            .bind(user_id)
            .bind(Utc::now())
            .map(|row: sqlx::sqlite::SqliteRow| models::reminder::Reminder {
                id: row.try_get("id").unwrap_or(-1),
                user_app_id: row.try_get("user_app_id").unwrap_or_default(),
                body: row.try_get("body").unwrap_or_default(),
                execution_id: row.try_get("execution_id").unwrap_or_default(),
                notification_type:
                    serde_json::from_str::<models::reminder::ReminderNotificationType>(
                        row.try_get("notification_type").unwrap_or_default(),
                    )
                    .unwrap_or_default(),
                send_at: row.try_get("send_at").unwrap_or_default(),
                user_timezone: row.try_get("user_timezone").unwrap_or_default(),
                created_at: row.try_get("created_at").unwrap_or_default(),
            })
            .fetch_all(&self.db_pool)
            .await?);
    }

    async fn get_reminder_execution_id(
        &self,
        user_id: i64,
        reminder_id: i64,
    ) -> anyhow::Result<Option<String>> {
        let execution_id: Option<String> = sqlx::query_scalar(
            "SELECT execution_id FROM reminder WHERE id=$1 AND user_app_id=$2 LIMIT 1;",
        )
        .bind(reminder_id)
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(execution_id)
    }

    async fn insert_user_remider(
        &self,
        reminder: &models::reminder::Reminder,
    ) -> anyhow::Result<i64> {
        Ok(sqlx::query(sqlite_queries::QUERY_INSERT_USER_REMINDER)
            .bind(reminder.user_app_id)
            .bind(reminder.body.to_string())
            .bind(reminder.execution_id.to_string())
            .bind(reminder.notification_type.to_string())
            .bind(reminder.send_at)
            .bind(reminder.user_timezone.to_string())
            .bind(reminder.created_at)
            .execute(&self.db_pool)
            .await?
            .last_insert_rowid())
    }

    async fn delete_user_reminder(&self, reminder_id: i64, user_id: i64) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM reminder WHERE id=$1 AND user_app_id=$2")
            .bind(reminder_id)
            .bind(user_id)
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }
}
