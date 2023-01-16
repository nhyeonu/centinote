use actix_web::{Error, error::{ErrorInternalServerError}};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use super::Entry;
use super::utils;

async fn create_entry_sqlx(
    db_pool: &PgPool,
    timezone_offset: i32,
    user_uuid: &str,
    title: &str,
    body: &str) -> Result<Entry, sqlx::Error>
{
    let entry_uuid = Uuid::new_v4().to_string();
    let current_timestamp_naive = Utc::now().naive_utc();
    let insert_result = sqlx::query("INSERT INTO journals VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(&entry_uuid)
        .bind(&user_uuid)
        .bind(&current_timestamp_naive)
        .bind(&timezone_offset)
        .bind(title)
        .bind(body)
        .execute(db_pool)
        .await;

    let current_timestamp_offset = utils::naive_to_offset(current_timestamp_naive, timezone_offset);

    match insert_result {
        Ok(_) => {
            let entry = Entry {
                created: current_timestamp_offset.to_rfc3339(),
                title: title.to_string(),
                body: body.to_string(),
                uuid: entry_uuid,
                user_uuid: user_uuid.to_string(),
            };

            Ok(entry)
        },
        Err(error) => return Err(error)
    }
}

impl Entry {
    pub async fn create(
        db_pool: &PgPool,
        timezone_offset: i32,
        user_uuid: &str,
        title: &str,
        body: &str) -> Result<Self, Error>
    {
        match create_entry_sqlx(db_pool, timezone_offset, user_uuid, title, body).await {
            Ok(value) => Ok(value),
            Err(error) => {
                println!("{error}");
                Err(ErrorInternalServerError("Database error."))
            }
        }
    }
}

