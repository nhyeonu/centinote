use actix_web::{Error, error::{ErrorNotFound, ErrorInternalServerError}};
use sqlx::{Row, PgPool};
use super::User;

async fn by_username_sqlx(
    db_pool: &PgPool,
    username: &str) -> Result<User, sqlx::Error> 
{
    let user_row = 
        sqlx::query("SELECT password_hash, uuid FROM users WHERE username = $1")
        .bind(username)
        .fetch_one(db_pool)
        .await?;

    let user_uuid: String = user_row.try_get("uuid")?;
    let password_hash: String = user_row.try_get("password_hash")?;

    Ok(User {
        uuid: user_uuid,
        username: username.to_string(),
        password_hash: password_hash,
    })
}

impl User {
    pub async fn by_username(
        db_pool: &PgPool,
        username: &str) -> Result<User, Error> 
    {
        match by_username_sqlx(db_pool, username).await {
            Ok(value) => Ok(value),
            Err(error) => {
                match error {
                    sqlx::Error::RowNotFound => Err(ErrorNotFound("User not found.")),
                    _ => Err(ErrorInternalServerError("Database error."))
                }
            }
        }
    }
}
