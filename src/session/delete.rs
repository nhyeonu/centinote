use actix_web::error::{Error, ErrorInternalServerError, ErrorNotFound};
use sqlx::PgPool;
use super::Session;

async fn delete_session(
    db_pool: &PgPool,
    session_uuid: &str,
    user_uuid: &str) -> Result<(), Error>
{
    let delete_result = 
        sqlx::query("DELETE FROM sessions WHERE uuid = $1 AND user_uuid = $2")
        .bind(session_uuid)
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

impl Session {
    pub async fn delete(self, db_pool: &PgPool) -> Result<Self, Error> {
        delete_session(db_pool, &self.uuid, &self.user_uuid).await?;
        Ok(self)
    }
}
