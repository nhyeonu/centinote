use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use actix_web::{post, delete, web, HttpRequest, HttpResponse, Responder};
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
}

fn verify_password(
    argon2: &Argon2<'_>,
    password: &str,
    password_hash: &str) -> Result<(), password_hash::errors::Error> 
{
    let parsed_hash = PasswordHash::new(password_hash)?;
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash)?)
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

#[post("/api/login")]
async fn post_login(
    data: web::Data<State<'_>>,
    info: web::Json<Login>) -> impl Responder 
{
    let user = match User::from_username(&info.username, &data.db_pool).await {
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

    match verify_password(&data.argon2, &info.password, &user.password_hash) {
        Ok(_) => {
            // Password is correct. Trying to create an access token.
            match create_auth_cookie(&data.db_pool, user.uuid.clone()).await {
                Ok(token) => return HttpResponse::Ok()
                    .cookie(token)
                    .cookie(Cookie::build("user_uuid", user.uuid)
                            .same_site(SameSite::Strict)
                            .path("/")
                            .finish())
                    .finish(),
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
