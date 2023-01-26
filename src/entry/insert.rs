use actix_web::{Error, error::{ErrorBadRequest, ErrorConflict, ErrorInternalServerError}};
use chrono::{DateTime, FixedOffset, Offset};
use sqlx::{PgPool, Row};
use uuid::{Uuid, Variant};
use super::Entry;

async fn count_entry_sqlx(
    db_pool: &PgPool,
    uuid: &str,
    user_uuid: &str) -> Result<i64, sqlx::Error> 
{
    let count: i64 = 
        sqlx::query("SELECT COUNT(*) AS count FROM journals WHERE uuid = $1 AND user_uuid = $2")
        .bind(uuid)
        .bind(user_uuid)
        .fetch_one(db_pool)
        .await?
        .try_get("count")?;

    Ok(count)
}

async fn insert_entry_sqlx(
    db_pool: &PgPool,
    uuid: &str,
    user_uuid: &str,
    created: DateTime<FixedOffset>,
    title: &str,
    body: &str) -> Result<Entry, sqlx::Error>
{
    sqlx::query("INSERT INTO journals VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(uuid)
        .bind(user_uuid)
        .bind(created.naive_utc())
        .bind(created.offset().fix().utc_minus_local() / 60)
        .bind(title)
        .bind(body)
        .execute(db_pool)
        .await?;

    Ok(Entry {
        created: created.to_string(),
        title: title.to_string(),
        body: body.to_string(),
        uuid: uuid.to_string(),
        user_uuid: user_uuid.to_string(),
    })
}

impl Entry {
    pub async fn insert(
        db_pool: &PgPool,
        uuid: &str,
        user_uuid: &str,
        created: &str,
        title: &str,
        body: &str) -> Result<Self, Error>
    {
        let entry_uuid = match Uuid::try_parse(uuid) {
            Ok(value) => value,
            Err(_) => return Err(ErrorBadRequest("UUID did not parse."))
        };

        if entry_uuid.get_variant() != Variant::RFC4122 {
            return Err(ErrorBadRequest("UUID is not RFC4122."));
        }

        if entry_uuid.get_version_num() != 4 {
            return Err(ErrorBadRequest("UUID is not v4."));
        }

        let created_datetime = match DateTime::parse_from_rfc3339(created) {
            Ok(value) => value,
            Err(_) => return Err(ErrorBadRequest("Timestamp did not parse."))
        };

        let count = match count_entry_sqlx(db_pool, uuid, user_uuid).await {
            Ok(value) => value,
            Err(error) => {
                println!("{error}");
                return Err(ErrorInternalServerError("Database error."))
            }
        };

        if count != 0 {
            return Err(ErrorConflict("Entry already exists."));
        }

        match insert_entry_sqlx(db_pool, uuid, user_uuid, created_datetime, title, body).await {
            Ok(value) => Ok(value),
            Err(error) => {
                println!("{error}");
                Err(ErrorInternalServerError("Database error."))
            }
        }
    }
}

