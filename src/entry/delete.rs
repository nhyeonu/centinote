use actix_web::{Error, error::{ErrorInternalServerError, ErrorNotFound}};
use sqlx::PgPool;
use super::Entry;

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
