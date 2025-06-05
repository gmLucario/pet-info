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

impl FromRow<'_, SqliteRow> for models::payment::Payment {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        Ok(Self {
            user_id: row.try_get("user_id")?,
            mp_paym_id: from_str::<usize>(row.try_get::<&str, &str>("mp_paym_id")?)
                .unwrap_or_default(),
            payment_idempotency_h: row.try_get("payment_idempotency_h")?,
            transaction_amount: row.try_get("transaction_amount")?,
            installments: row.try_get("installments")?,
            payment_method_id: row.try_get("payment_method_id")?,
            issuer_id: row.try_get("issuer_id")?,
            status: serde_json::from_str::<models::payment::PaymentStatus>(&format!(
                "\"{}\"",
                row.try_get::<String, &str>("status")?
            ))
            .unwrap_or_default(),
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
        Ok(sqlx::query(sqlite_queries::QUERY_GET_USER_APP_BY_EMAIL)
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
            .await?)
    }

    async fn insert_verified_phone_to_user_app(
        &self,
        user_app_id: i64,
        phone: &str,
    ) -> anyhow::Result<()> {
        Ok(
            sqlx::query("UPDATE user_app SET phone_reminder=$1, updated_at=$2 WHERE id=$3;")
                .bind(phone)
                .bind(Utc::now())
                .bind(user_app_id)
                .execute(&self.db_pool)
                .await
                .map(|_| ())?,
        )
    }

    async fn set_to_null_verified_phone(&self, user_app_id: i64) -> anyhow::Result<()> {
        Ok(
            sqlx::query("UPDATE user_app SET phone_reminder=NULL, updated_at=$1 WHERE id=$2;")
                .bind(Utc::now())
                .bind(user_app_id)
                .execute(&self.db_pool)
                .await
                .map(|_| ())?,
        )
    }

    async fn insert_user_app(&self, app_user: &models::user_app::User) -> anyhow::Result<i64> {
        let mut transaction = self.db_pool.begin().await?;

        let user_app_id = sqlx::query(
            "INSERT INTO user_app(email,account_role,created_at,updated_at) VALUES($1,$2,$3,$4);",
        )
        .bind(&app_user.email)
        .bind(app_user.account_role.to_string())
        .bind(app_user.created_at)
        .bind(app_user.updated_at)
        .execute(&mut *transaction)
        .await?
        .last_insert_rowid();

        sqlx::query("INSERT INTO add_pet_balance(user_id, balance) VALUES($1, 0);")
            .bind(user_app_id)
            .execute(&mut *transaction)
            .await?;

        transaction.commit().await?;

        Ok(user_app_id)
    }

    async fn set_pet_balance(&self, user_id: i64, balance: u32) -> anyhow::Result<()> {
        Ok(
            sqlx::query("UPDATE add_pet_balance SET balance=$2 WHERE user_id = $1;")
                .bind(user_id)
                .bind(balance)
                .execute(&self.db_pool)
                .await
                .map(|_| ())?,
        )
    }

    async fn get_pet_balance(&self, user_id: i64) -> anyhow::Result<u32> {
        Ok(
            sqlx::query_scalar("SELECT balance FROM add_pet_balance WHERE user_id = $1;")
                .bind(user_id)
                .fetch_one(&self.db_pool)
                .await?,
        )
    }

    async fn is_pet_external_id_linked(
        &self,
        pet_external_id: &Uuid,
    ) -> anyhow::Result<Option<bool>> {
        Ok(
            sqlx::query_scalar(sqlite_queries::QUERY_IS_PET_EXTERNAL_ID_LINKED)
                .bind(pet_external_id.to_string())
                .fetch_optional(&self.db_pool)
                .await?,
        )
    }

    /// Retrieves the user payments DESC order
    async fn get_user_payments(
        &self,
        user_id: i64,
        status: Option<models::payment::PaymentStatus>,
    ) -> anyhow::Result<Vec<models::payment::Payment>> {
        let status = status.map(|s| s.to_string()).unwrap_or("all".into());

        Ok(
            sqlx::query_as::<_, models::payment::Payment>(sqlite_queries::QUERY_GET_USER_PAYM_SUBS)
                .bind(user_id)
                .bind(&status)
                .fetch_all(&self.db_pool)
                .await?,
        )
    }

    async fn save_subs_payment(&self, payment: &models::payment::Payment) -> anyhow::Result<i64> {
        Ok(sqlx::query(sqlite_queries::QUERY_INSERT_NEW_SUB_PAYM)
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
            .last_insert_rowid())
    }

    async fn set_user_as_subscribed(&self, user_id: i64) -> anyhow::Result<()> {
        Ok(sqlx::query(
            "UPDATE user_app SET is_subscribed=1, updated_at=$1 WHERE id = $2 AND is_subscribed=0;",
        )
        .bind(Utc::now())
        .bind(user_id)
        .execute(&self.db_pool)
        .await
        .map(|_| ())?)
    }

    async fn save_pet(&self, pet: &models::pet::Pet) -> anyhow::Result<i64> {
        let mut transaction = self.db_pool.begin().await?;

        let id_external_id = if let Some(id) = sqlx::query_scalar::<_, i64>(
            "SELECT peid.id FROM pet_external_id AS peid WHERE peid.external_id = $1;",
        )
        .bind(pet.external_id.to_string())
        .fetch_optional(&mut *transaction)
        .await?
        {
            id
        } else {
            sqlx::query(sqlite_queries::QUERY_INSERT_PET_EXTERNAL_ID)
                .bind(pet.external_id.to_string())
                .bind(chrono::Utc::now())
                .execute(&mut *transaction)
                .await?
                .last_insert_rowid()
        };

        let pet_id = sqlx::query(sqlite_queries::QUERY_INSERT_PET)
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
            .execute(&mut *transaction)
            .await?
            .last_insert_rowid();

        sqlx::query(sqlite_queries::QUERY_LINK_PET_WITH_EXTERNAL_ID)
            .bind(pet_id)
            .bind(id_external_id)
            .execute(&mut *transaction)
            .await?;

        transaction.commit().await?;

        return Ok(pet_id);
    }

    async fn update_pet(&self, pet: &models::pet::Pet) -> anyhow::Result<i64> {
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
        Ok(sqlx::query(sqlite_queries::QUERY_DELETE_PET)
            .bind(pet_id)
            .bind(user_id)
            .execute(&self.db_pool)
            .await
            .map(|_| ())?)
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
        Ok(
            sqlx::query_scalar::<_, String>("SELECT p.pic FROM pet AS p WHERE p.external_id = $1;")
                .bind(pet_external_id.to_string())
                .fetch_optional(&self.db_pool)
                .await?,
        )
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

        Ok(
            sqlx::query_as::<_, models::pet::PetWeight>(sqlite_queries::QUERY_INSERT_PET_WEIGHT)
                .bind(pet_external_id.to_string())
                .bind(user_id)
                .bind(weight)
                .bind(date)
                .fetch_one(&self.db_pool)
                .await?,
        )
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
        Ok(sqlx::query_as::<_, models::user_app::OwnerContact>(
            sqlite_queries::QUERY_GET_PET_OWNER_CONTACTS,
        )
        .bind(pet_external_id.to_string())
        .fetch_all(&self.db_pool)
        .await?)
    }

    async fn get_owner_contacts(
        &self,
        user_id: i64,
    ) -> anyhow::Result<Vec<models::user_app::OwnerContact>> {
        Ok(sqlx::query_as::<_, models::user_app::OwnerContact>(
            sqlite_queries::QUERY_GET_OWNER_CONTACTS,
        )
        .bind(user_id)
        .fetch_all(&self.db_pool)
        .await?)
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
        Ok(
            sqlx::query_as::<_, models::pet::PetNote>(sqlite_queries::QUERY_GET_PET_NOTES)
                .bind(pet_id)
                .bind(user_id)
                .fetch_all(&self.db_pool)
                .await?,
        )
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
        Ok(sqlx::query(sqlite_queries::QUERY_GET_USER_ACTIVE_REMINDERS)
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
            .await?)
    }

    async fn get_reminder_execution_id(
        &self,
        user_id: i64,
        reminder_id: i64,
    ) -> anyhow::Result<Option<String>> {
        Ok(sqlx::query_scalar::<_, String>(
            "SELECT execution_id FROM reminder WHERE id=$1 AND user_app_id=$2 LIMIT 1;",
        )
        .bind(reminder_id)
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await?)
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
