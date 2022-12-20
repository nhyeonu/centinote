use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use actix_web::{post, web, HttpResponse, Responder};
use actix_web::cookie::{Cookie, SameSite};
use serde::Deserialize;
use sqlx::Row;
use argon2::password_hash::{PasswordHash, PasswordVerifier};
use crate::state::State;

#[derive(Deserialize)]
struct Login {
    username: String,
    password: String,
}

#[post("/login")]
async fn post(data: web::Data<State<'_>>, info: web::Json<Login>) -> impl Responder {
    let password_select_result = sqlx::query("SELECT HashedPassword AS password_hash, Uuid AS uuid FROM users WHERE Username = $1")
        .bind(info.username.clone())
        .fetch_one(&data.db_pool)
        .await;

    let user_row = match password_select_result {
        Ok(row) => row,
        Err(error) => {
            match error {
                sqlx::Error::RowNotFound => {
                    return HttpResponse::Unauthorized().finish(); 
                },
                _ => {
                    return HttpResponse::InternalServerError().finish(); 
                }
            }
        }
    };

    let password_hash: String = match user_row.try_get("password_hash") {
        Ok(hash) => hash,
        Err(error) => {
            println!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let parsed_hash = match PasswordHash::new(&password_hash) {
        Ok(parsed) => parsed,
        Err(error) => {
            println!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    //TODO: Separate out content of this if statement to a function.
    if data.argon2.verify_password(info.password.as_bytes(), &parsed_hash).is_ok() {
        let user_uuid: String = match user_row.try_get("uuid") {
            Ok(uuid) => uuid,
            Err(error) => {
                println!("{}", error);
                return HttpResponse::InternalServerError().finish();
            }
        };

        let token: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        let insert_result = sqlx::query("INSERT INTO sessions VALUES ($1, $2);")
            .bind(user_uuid)
            .bind(token.clone())
            .execute(&data.db_pool)
            .await;

        match insert_result {
            Ok(_) => {
                let cookie = Cookie::build("auth", token)
                    .http_only(true)
                    .same_site(SameSite::Strict)
                    .finish();

                return HttpResponse::Found().cookie(cookie).finish();
            },
            Err(error) => {
                println!("{}", error);
                return HttpResponse::InternalServerError().finish();
            }
        }
    } else {
        return HttpResponse::Unauthorized().finish(); 
    }
}
