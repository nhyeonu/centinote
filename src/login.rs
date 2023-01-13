use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use actix_web::{
    post, delete, web, HttpRequest, HttpResponse, Responder, Error, 
    error::{ErrorUnauthorized, ErrorInternalServerError}
};
use actix_web::cookie::{Cookie, SameSite};
use serde::Deserialize;
use sqlx::{PgPool, Row};
use chrono::{Utc, Duration};
use argon2::Argon2;
use argon2::password_hash;
use argon2::password_hash::{PasswordHash, PasswordVerifier};
use crate::state::State;
use crate::utils;

#[derive(Deserialize)]
struct Login {
    username: String,
    password: String,
}

struct User {
    uuid: String,
    username: String,
    password_hash: String,
}

impl User {
    async fn from_username(username: &str, db_pool: &PgPool) -> Result<User, sqlx::Error> {
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

    fn verify_password(&self, password: &str) -> Result<(), password_hash::errors::Error> 
    {
        let parsed_hash = PasswordHash::new(&self.password_hash)?;
        Argon2::default().verify_password(password.as_bytes(), &parsed_hash)?;
        Ok(())
    }
}

async fn create_auth_cookie(
    db_pool: &PgPool,
    user_uuid: String) -> Result<Cookie, sqlx::Error> 
{
    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    sqlx::query("INSERT INTO sessions VALUES ($1, $2, $3);")
        .bind(user_uuid)
        .bind(Utc::now().naive_utc() + Duration::minutes(30))
        .bind(&token)
        .execute(db_pool)
        .await?;

    let cookie = Cookie::build("auth", token.clone())
        .http_only(true)
        .path("/")
        .same_site(SameSite::Strict)
        .finish();

    Ok(cookie)
}

async fn get_user(username: &str, db_pool: &PgPool) -> Result<User, Error> {
    match User::from_username(username, db_pool).await {
        Ok(value) => Ok(value),
        Err(error) => {
            match error {
                // Unauthorized when user is not found.
                sqlx::Error::RowNotFound => Err(ErrorUnauthorized("User is not found.")),
                // Something else has gone wrong.
                _ => {
                    println!("{}", error);
                    Err(ErrorInternalServerError("Database error."))
                }
            }
        }
    }
}

fn verify_password_actix(user: &User, password: &str) -> Result<(), Error> {
    match user.verify_password(password) {
        Ok(value) => Ok(value),
        Err(error) => {
            match error {
                // Unauthorized when password is wrong.
                password_hash::errors::Error::Password => {
                    Err(ErrorUnauthorized("Password is wrong."))
                },
                // Something else has gone wrong when parsing or verifying password hash.
                _ => {
                    Err(ErrorInternalServerError("Password verification failed."))
                }
            }
        }
    }
}

#[post("/api/login")]
async fn post_login(
    data: web::Data<State<'_>>,
    info: web::Json<Login>) -> Result<HttpResponse, Error>
{
    let user = get_user(&info.username, &data.db_pool).await?;
    verify_password_actix(&user, &info.password)?;

    let auth = match create_auth_cookie(&data.db_pool, user.uuid.clone()).await {
        Ok(value) => value,
        Err(error) => {
            println!("{}", error);
            return Err(ErrorInternalServerError("Database error."));
        }
    };

    let uuid = Cookie::build("user_uuid", user.uuid)
        .same_site(SameSite::Strict)
        .path("/")
        .finish();

    Ok(HttpResponse::Ok().cookie(auth).cookie(uuid).finish())
}

#[post("/api/session")]
async fn post_session(
    data: web::Data<State<'_>>,
    req: HttpRequest) -> impl Responder
{
    let user_uuid = utils::verify_request_token!(&data.db_pool, &req);
    let token = utils::get_auth_token(&req).unwrap();

    let update_result = 
        sqlx::query("UPDATE sessions SET expiry = $1 WHERE token = $2 AND user_uuid = $3")
        .bind(Utc::now().naive_utc() + Duration::minutes(30))
        .bind(&token)
        .bind(&user_uuid)
        .execute(&data.db_pool)
        .await;

    match update_result {
        Ok(_) => return HttpResponse::Ok().finish(),
        Err(error) => {
            println!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
    }
}

#[delete("/api/session")]
async fn delete_session(
    data: web::Data<State<'_>>,
    req: HttpRequest) -> impl Responder
{
    let user_uuid = utils::verify_request_token!(&data.db_pool, &req);
    let token = utils::get_auth_token(&req).unwrap();

    let delete_result = 
        sqlx::query("DELETE FROM sessions WHERE token = $1 AND user_uuid = $2")
        .bind(&token)
        .bind(&user_uuid)
        .execute(&data.db_pool)
        .await;

    let mut user_uuid_cookie = Cookie::build("user_uuid", "")
        .same_site(SameSite::Strict)
        .path("/")
        .finish();
    user_uuid_cookie.make_removal();
    
    let mut auth_cookie = Cookie::build("auth", "")
        .same_site(SameSite::Strict)
        .path("/")
        .finish();
    auth_cookie.make_removal();

    match delete_result {
        Ok(_) => return HttpResponse::Ok()
            .cookie(user_uuid_cookie)
            .cookie(auth_cookie)
            .finish(),
        Err(error) => {
            println!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
    }
}
