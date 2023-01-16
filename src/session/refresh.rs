use actix_web::error::{Error, ErrorInternalServerError};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use super::Session;

async fn refresh_auth_token(
    db_pool: &PgPool,
    session_uuid: &str,
    user_uuid: &str) -> Result<(), Error>
{
    let update_result = 
        sqlx::query("UPDATE sessions SET expiry = $1 WHERE uuid = $2 AND user_uuid = $3")
        .bind(Utc::now().naive_utc() + Duration::minutes(30))
        .bind(session_uuid)
        .bind(user_uuid)
        .execute(db_pool)
        .await;

    match update_result {
        Ok(_) => Ok(()),
        Err(error) => {
            println!("{}", error);
            Err(ErrorInternalServerError("Database error."))
        }
    }
}

impl Session {
    pub async fn refresh(self, db_pool: &PgPool) -> Result<Self, Error> {
        refresh_auth_token(db_pool, &self.uuid, &self.user_uuid).await?;
        Ok(self)
    }
}
