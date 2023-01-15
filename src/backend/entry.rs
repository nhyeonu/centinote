use actix_web::{Error, error::{ErrorInternalServerError, ErrorNotFound}};
use chrono::{DateTime, NaiveDateTime, FixedOffset, TimeZone, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

fn naive_to_offset(
    naive: NaiveDateTime,
    timezone_offset: i32) -> DateTime<FixedOffset> 
{
    let offset = match FixedOffset::west_opt(timezone_offset * 60) {
        Some(value) => value,
        None => FixedOffset::west_opt(0).unwrap()
    };

    offset.from_utc_datetime(&naive)
}

pub struct Entry {
    pub created: String,
    pub title: String,
    pub body: String,
    pub uuid: String,
    pub user_uuid: String,
}

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

    let current_timestamp_offset = naive_to_offset(current_timestamp_naive, timezone_offset);

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

async fn update_entry(
    db_pool: &PgPool,
    entry_uuid: &str,
    user_uuid: &str,
    title: &str,
    body: &str) -> Result<(), Error>
{
    let update_result = 
        sqlx::query("UPDATE journals SET title = $1, body = $2 WHERE uuid = $3 AND user_uuid = $4")
        .bind(title)
        .bind(body)
        .bind(entry_uuid)
        .bind(user_uuid)
        .execute(db_pool)
        .await;

    let query_result = match update_result {
        Ok(value) => value,
        Err(error) => {
            println!("{error}");
            return Err(ErrorInternalServerError("Database error."));
        }
    };

    if query_result.rows_affected() == 0 {
        return Err(ErrorNotFound("Entry not found."));
    }

    Ok(())
}

impl Entry {
    pub async fn update(
        mut self,
        db_pool: &PgPool,
        title: &str,
        body: &str) -> Result<Self, Error>
    {
        update_entry(db_pool, &self.uuid, &self.user_uuid, title, body).await?;

        self.title = title.to_string();
        self.body = body.to_string();

        Ok(self)
    }
}

async fn delete_entry(
    db_pool: &PgPool,
    entry_uuid: &str,
    user_uuid: &str) -> Result<(), Error>
{
    let delete_result = sqlx::query("DELETE FROM journals WHERE uuid = $1 AND user_uuid = $2")
        .bind(entry_uuid)
        .bind(user_uuid)
        .execute(db_pool)
        .await;

    let query_result = match delete_result {
        Ok(value) => value,
        Err(error) => {
            println!("{error}");
            return Err(ErrorInternalServerError("Database error."));
        }
    };

    if query_result.rows_affected() == 0 {
        return Err(ErrorNotFound("Entry not found."));
    }

    Ok(())
}

impl Entry {
    pub async fn delete(self, db_pool: &PgPool) -> Result<Self, Error> {
        delete_entry(db_pool, &self.uuid, &self.user_uuid).await?;
        Ok(self)
    }
}

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

        naive_to_offset(created_utc_naive, timezone_offset_minute)
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

async fn uuids_by_user_sqlx(
    db_pool: &PgPool,
    user_uuid: &str) -> Result<Vec<String>, sqlx::Error>
{
    let rows = 
        sqlx::query("SELECT uuid FROM journals WHERE user_uuid = $1 ORDER BY created DESC")
        .bind(user_uuid)
        .fetch_all(db_pool)
        .await?;

    let uuid_results: Vec<Result<String, sqlx::Error>> 
        = rows.iter().map(|row| row.try_get("uuid")).collect();

    let uuids_result: Result<Vec<String>, sqlx::Error> 
        = uuid_results.into_iter().collect();

    let uuids: Vec<String> = uuids_result?;

    Ok(uuids)
}

impl Entry {
    pub async fn uuids_by_user(
        db_pool: &PgPool,
        user_uuid: &str) -> Result<Vec<String>, Error> 
    {
        match uuids_by_user_sqlx(db_pool, user_uuid).await {
            Ok(value) => Ok(value),
            Err(error) => {
                println!("{error}");
                Err(ErrorInternalServerError("Database error."))
            }
        }
    }
}
