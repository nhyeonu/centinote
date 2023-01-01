use sqlx::{PgPool, Row};
use actix_web::HttpRequest;
use chrono::{Utc, NaiveDateTime};

pub async fn verify_token(
    pool: &PgPool,
    auth_token: &str) -> Result<String, sqlx::Error> 
{
    let user_row = 
        sqlx::query("SELECT user_uuid, expiry FROM sessions WHERE token = $1")
        .bind(auth_token)
        .fetch_one(pool)
        .await?;

    let user_uuid: String = user_row.try_get("user_uuid")?;
    let expiry: NaiveDateTime = user_row.try_get("expiry")?;

    if expiry.timestamp() < Utc::now().naive_utc().timestamp() {
        sqlx::query("DELETE FROM sessions WHERE token = $1")
            .bind(auth_token)
            .execute(pool)
            .await?;

        return Err(sqlx::Error::RowNotFound);
    }

    Ok(user_uuid)
}

pub fn get_auth_token(req: &HttpRequest) -> Option<String> {
    match req.cookie("auth") {
        Some(cookie) => Some(cookie.value().to_string()),
        None => None
    }
}

macro_rules! verify_request_token {
    ($pool:expr, $req:expr) => {
        { 
            let auth_token = match crate::utils::get_auth_token($req) {
                Some(token) => token,
                None => return HttpResponse::Unauthorized().finish()
            };

            let user_uuid = match crate::utils::verify_token($pool, &auth_token).await {
                Ok(value) => value,
                Err(error) => {
                    match error {
                        sqlx::Error::RowNotFound => return HttpResponse::Unauthorized().finish(),
                        _ => {
                            println!("{}", error);
                            return HttpResponse::InternalServerError().finish();
                        }
                    }
                }
            };

            user_uuid
        }
    }
}
pub(crate) use verify_request_token;
