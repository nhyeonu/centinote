use actix_web::{Error, error::{ErrorInternalServerError, ErrorNotFound}};
use sqlx::PgPool;
use super::Entry;

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
