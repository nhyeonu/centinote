use actix_web::{Error, error::ErrorInternalServerError};
use sqlx::{PgPool, Row};
use super::Entry;

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
