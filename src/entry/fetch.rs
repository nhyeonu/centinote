use actix_web::{Error, error::{ErrorInternalServerError, ErrorNotFound}};
use chrono::NaiveDateTime;
use sqlx::{PgPool, Row};
use super::Entry;
use super::utils;

async fn by_uuid_and_user_sqlx(
    db_pool: &PgPool,
    entry_uuid: &str,
    user_uuid: &str) -> Result<Entry, sqlx::Error>
{
    let entry_row = 
        sqlx::query("SELECT * FROM journals WHERE uuid = $1 AND user_uuid = $2")
        .bind(entry_uuid)
        .bind(user_uuid)
        .fetch_one(db_pool)
        .await?;

    let created = {
        let created_utc_naive: NaiveDateTime = entry_row.try_get("created")?;
        let timezone_offset_minute: i32 = entry_row.try_get("timezone_offset")?;

        utils::naive_to_offset(created_utc_naive, timezone_offset_minute)
    };

    let title: String = entry_row.try_get("title")?;
    let body: String = entry_row.try_get("body")?;

    Ok(Entry {
        created: created.to_rfc3339(),
        title: title,
        body: body,
        uuid: entry_uuid.to_string(),
        user_uuid: user_uuid.to_string(),
    })
}

impl Entry {
    pub async fn by_uuid_and_user(
        db_pool: &PgPool,
        entry_uuid: &str,
        user_uuid: &str) -> Result<Self, Error> 
    {
        match by_uuid_and_user_sqlx(db_pool, entry_uuid, user_uuid).await {
            Ok(value) => Ok(value),
            Err(error) => {
                match error {
                    sqlx::Error::RowNotFound => Err(ErrorNotFound("Entry not found.")),
                    _ => {
                        println!("{error}");
                        Err(ErrorInternalServerError("Database error."))
                    }
                }
            }
        }
    }
}
