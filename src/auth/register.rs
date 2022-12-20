use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::Row;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHasher, SaltString};
use uuid::Uuid;
use crate::state::State;

#[derive(Deserialize)]
struct Register {
    username: String,
    password: String,
}

#[post("/register")]
async fn post(data: web::Data<State<'_>>, info: web::Json<Register>) -> impl Responder {
    // len() returns number of bytes in a string and VARCHAR in SQL also counts length in bytes.
    // Username length limit must be equal to the length limit defined by scripts at sql/migrations
    if info.username.len() > 64 || info.password.len() > 64 {
        return HttpResponse::BadRequest().finish();
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = match data.argon2.hash_password(info.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(error) => {
            println!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let user_count_result = sqlx::query("SELECT COUNT(*) AS count FROM users WHERE Username = $1")
        .bind(info.username.clone())
        .fetch_one(&data.db_pool)
        .await;

    let user_count_row = match user_count_result {
        Ok(row) => row,
        Err(error) => {
            println!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let user_count: i64 = match user_count_row.try_get("count") {
        Ok(count) => count,
        Err(error) => {
            println!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    if user_count == 0 {
        let insert_request = sqlx::query("INSERT INTO users VALUES ($1, $2, $3);")
            .bind(Uuid::new_v4().to_string())
            .bind(info.username.clone())
            .bind(password_hash)
            .execute(&data.db_pool)
            .await;

        match insert_request {
            Ok(_) => return HttpResponse::Created().finish(),
            Err(error) => {
                println!("{}", error);
                return HttpResponse::InternalServerError().finish();
            }
        }
        
    } else {
        return HttpResponse::Conflict().finish();
    }
}
