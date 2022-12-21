use sqlx::{PgPool, Row};
use actix_web::{HttpRequest, HttpResponse};

pub async fn verify_session(pool: &PgPool, req: HttpRequest) -> Result<String, HttpResponse> {
    let auth_cookie = match req.cookie("auth") {
        Some(value) => value,
        None => { 
            return Err(HttpResponse::Unauthorized().finish()); 
        }
    };

    let uuid_select_result = sqlx::query("SELECT UserUuid AS uuid FROM sessions WHERE Token = $1")
        .bind(auth_cookie.value())
        .fetch_one(pool)
        .await;

    let uuid_row = match uuid_select_result {
        Ok(row) => row,
        Err(error) => {
            match error {
                sqlx::Error::RowNotFound => {
                    return Err(HttpResponse::Unauthorized().finish()); 
                },
                _ => {
                    println!("{}", error);
                    return Err(HttpResponse::InternalServerError().finish()); 
                }
            }
        }
    };

    let uuid: String = match uuid_row.try_get("uuid") {
        Ok(value) => value,
        Err(error) => {
            println!("{}", error);
            return Err(HttpResponse::InternalServerError().finish());
        }
    };

    Ok(uuid)
}

