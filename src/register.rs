use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::{PgPool, Row};
use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHasher, SaltString};
use uuid::Uuid;
use crate::state::State;

async fn create_user(db_pool: &PgPool, argon2: &Argon2<'_>, username: &str, password: &str) -> Result<String, HttpResponse> {
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(error) => {
            println!("{}", error);
            return Err(HttpResponse::InternalServerError().finish());
        }
    };

    let user_uuid = Uuid::new_v4().to_string();

    let insert_request = sqlx::query("INSERT INTO users VALUES ($1, $2, $3);")
        .bind(user_uuid.clone())
        .bind(username)
        .bind(password_hash)
        .execute(db_pool)
        .await;

    match insert_request {
        Ok(_) => return Ok(user_uuid),
        Err(error) => {
            println!("{}", error);
            return Err(HttpResponse::InternalServerError().finish());
        }
    }
}

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

    let user_count_result = sqlx::query("SELECT COUNT(*) AS count FROM users WHERE username = $1")
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
        match create_user(&data.db_pool, &data.argon2, &info.username, &info.password).await {
            Ok(_uuid) => return HttpResponse::Ok().finish(),
            Err(error) => return error
        }
    } else {
        return HttpResponse::Conflict().finish();
    }
}
