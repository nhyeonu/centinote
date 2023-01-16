use rand::rngs::OsRng;
use actix_web::{Error, error::{ErrorInternalServerError, ErrorConflict}};
use sqlx::{Row, PgPool};
use argon2::{Argon2, password_hash::{PasswordHasher, SaltString}};
use uuid::Uuid;
use super::User;

fn hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    match Argon2::default().hash_password(password.as_bytes(), &salt) {
        Ok(hash) => Ok(hash.to_string()),
        Err(error) => {
            println!("{}", error);
            Err(ErrorInternalServerError("Failed to hash password."))
        }
    }
}

async fn insert_user(
    db_pool: &PgPool,
    uuid: &str,
    username: &str,
    password_hash: &str) -> Result<(), Error> 
{
    let insert_result = sqlx::query("INSERT INTO users VALUES ($1, $2, $3);")
        .bind(uuid)
        .bind(username)
        .bind(password_hash)
        .execute(db_pool)
        .await;

    match insert_result {
        Ok(_) => Ok(()),
        Err(error) => {
            println!("{}", error);
            Err(ErrorInternalServerError("Database error."))
        }
    }
}

async fn count_user_by_username_sqlx(
    db_pool: &PgPool,
    username: &str) -> Result<i64, sqlx::Error>
{
    let user_count_row = sqlx::query("SELECT COUNT(*) AS count FROM users WHERE username = $1")
        .bind(username)
        .fetch_one(db_pool)
        .await?;

    let user_count: i64 = user_count_row.try_get("count")?;
    Ok(user_count)
}

impl User {
    pub async fn create(
        db_pool: &PgPool,
        username: &str,
        password: &str) -> Result<Self, Error> 
    {
        let user_count = match count_user_by_username_sqlx(db_pool, username).await {
            Ok(value) => value,
            Err(error) => {
                println!("{error}");
                return Err(ErrorInternalServerError("Database error."));
            }
        };

        if user_count != 0 {
            return Err(ErrorConflict("User already exists."));
        }

        let user_uuid = Uuid::new_v4().to_string();
        let password_hash = hash_password(&password)?;

        insert_user(db_pool, &user_uuid, &username, &password_hash).await?;

        Ok(User {
            uuid: user_uuid,
            username: username.to_string(),
            password_hash: password_hash,
        })
    }
}
