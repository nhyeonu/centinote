use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use sqlx::{PgPool, Row};
use serde::{Serialize, Deserialize};
use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHasher, SaltString};
use uuid::Uuid;
use crate::State;
use crate::utils;

#[derive(Deserialize)]
struct UserCreate {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct UserData {
    username: String,
}

async fn create_user(
    db_pool: &PgPool,
    argon2: &Argon2<'_>,
    username: &str,
    password: &str) -> Result<String, HttpResponse> 
{
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

#[post("/users")]
async fn post(
    data: web::Data<State<'_>>,
    info: web::Json<UserCreate>) -> impl Responder 
{
    // len() returns number of bytes in a string and VARCHAR in SQL also counts length in bytes.
    // Username length limit must be equal to the length limit defined by scripts at sql/migrations
    if info.username.len() > 64 || info.password.len() > 64 {
        return HttpResponse::BadRequest().finish();
    }

    if info.username.len() < 2 || info.password.len() < 6 {
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

#[get("/users/{user_uuid}")]
async fn get(
    data: web::Data<State<'_>>,
    req: HttpRequest,
    path: web::Path<String>) -> impl Responder 
{
    let user_uuid = {
        let request_user_uuid = path.into_inner();
        let trusted_user_uuid = utils::verify_request_token!(&data.db_pool, &req);

        if request_user_uuid != trusted_user_uuid {
            return HttpResponse::Unauthorized().finish();
        }

        trusted_user_uuid
    };

    let select_result = 
        sqlx::query("SELECT username FROM users WHERE uuid = $1")
        .bind(&user_uuid)
        .fetch_one(&data.db_pool)
        .await;

    let user_row = match select_result {
        Ok(row) => row,
        Err(error) => {
            match error {
                sqlx::Error::RowNotFound => return HttpResponse::NotFound().finish(),
                _ => {
                    println!("{}", error);
                    return HttpResponse::InternalServerError().finish();
                }
            }
        }
    };

    let username: String = match user_row.try_get("username") {
        Ok(uuid) => uuid,
        Err(error) => {
            println!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    web::Json(UserData {
        username: username,
    }).respond_to(&req).map_into_boxed_body()
}
