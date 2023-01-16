use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use actix_web::error::{Error, ErrorInternalServerError};
use sqlx::PgPool;
use chrono::{Duration, Utc};
use uuid::Uuid;
use super::Session;

async fn create_auth_token(
    db_pool: &PgPool,
    user_uuid: &str) -> Result<(String, String), sqlx::Error> 
{
    let uuid = Uuid::new_v4().to_string();
    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    sqlx::query("INSERT INTO sessions VALUES ($1, $2, $3, $4);")
        .bind(&uuid)
        .bind(user_uuid)
        .bind(Utc::now().naive_utc() + Duration::minutes(30))
        .bind(&token)
        .execute(db_pool)
        .await?;

    Ok((uuid, token))
}


impl Session {
    pub async fn create(db_pool: &PgPool, user_uuid: &str) -> Result<Self, Error> {
        match create_auth_token(db_pool, user_uuid).await {
            Ok((uuid, token)) => Ok(Session {
                uuid: uuid,
                user_uuid: user_uuid.to_string(),
                token: token
            }),
            Err(error) => {
                println!("{error}");
                Err(ErrorInternalServerError("Database error."))
            }
        }
    }
}
