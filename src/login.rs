use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use actix_web::{post, web, HttpResponse, Responder};
use actix_web::cookie::{Cookie, SameSite};
use serde::Deserialize;
use sqlx::{PgPool, Row};
use argon2::Argon2;
use argon2::password_hash;
use argon2::password_hash::{PasswordHash, PasswordVerifier};
use crate::state::State;

#[derive(Deserialize)]
struct Login {
    username: String,
    password: String,
}

async fn query_user_by_username(db_pool: &PgPool, username: &str) -> Result<(String, String), sqlx::Error> {
    let password_select_result = sqlx::query("SELECT HashedPassword AS password_hash, Uuid AS uuid FROM users WHERE Username = $1")
        .bind(username)
        .fetch_one(db_pool)
        .await;

    let user_row = match password_select_result {
        Ok(row) => row,
        Err(error) => return Err(error)
    };

    let user_uuid: String = match user_row.try_get("uuid") {
        Ok(uuid) => uuid,
        Err(error) => return Err(error)
    };

    let password_hash: String = match user_row.try_get("password_hash") {
        Ok(hash) => hash,
        Err(error) => return Err(error)
    };

    Ok((user_uuid, password_hash))
}

async fn verify_password(argon2: &Argon2<'_>, password: &str, password_hash: &str) -> Result<(), password_hash::errors::Error> {
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(parsed) => parsed,
        Err(error) => return Err(error)
    };

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(()),
        Err(error) => Err(error)
    }
}

async fn create_access_token(db_pool: &PgPool, user_uuid: String) -> Result<Cookie, sqlx::Error> {
    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    let insert_result = sqlx::query("INSERT INTO sessions VALUES ($1, $2);")
        .bind(user_uuid)
        .bind(&token)
        .execute(db_pool)
        .await;

    match insert_result {
        Ok(_) => {
            Ok(
                Cookie::build("auth", token.clone())
                .http_only(true)
                .same_site(SameSite::Strict)
                .finish()
            )
        },
        Err(error) => Err(error)
    }
}

#[post("/login")]
async fn post(data: web::Data<State<'_>>, info: web::Json<Login>) -> impl Responder {
    let (user_uuid, password_hash) = match query_user_by_username(&data.db_pool, &info.username).await {
        Ok(values) => values,
        Err(error) => {
            match error {
                // Unauthorized when user is not found.
                sqlx::Error::RowNotFound => return HttpResponse::Unauthorized().finish(),
                // Something else has gone wrong.
                _ => {
                    println!("{}", error);
                    return HttpResponse::InternalServerError().finish();
                }
            }
        }
    };

    match verify_password(&data.argon2, &info.password, &password_hash).await {
        Ok(_) => {
            // Password is correct. Trying to create an access token.
            match create_access_token(&data.db_pool, user_uuid).await {
                Ok(cookie) => return HttpResponse::Ok().cookie(cookie).finish(),
                Err(error) => {
                    println!("{}", error);
                    return HttpResponse::InternalServerError().finish();
                }
            }
        },
        Err(error) => {
            match error {
                // Unauthorized when password is wrong.
                password_hash::errors::Error::Password => {
                    return HttpResponse::Unauthorized().finish(); 
                },
                // Something else has gone wrong when parsing or verifying password hash.
                _ => {
                    return HttpResponse::InternalServerError().finish();
                }
            }
        }
    }
}
