use actix_web::{
    web, FromRequest, HttpRequest,
    cookie::Cookie,
    dev::{Payload, Path, Url},
    error::{Error, ErrorUnauthorized, ErrorInternalServerError}
};
use sqlx::{PgPool, Row};
use chrono::{NaiveDateTime, Utc};
use std::pin::Pin;
use std::future::Future;
use super::Session;

async fn get_session_details_sqlx(
    token: &str,
    db_pool: &PgPool) -> Result<(String, String, NaiveDateTime), sqlx::Error>
{
    let session_row = sqlx::query("SELECT uuid, user_uuid, expiry FROM sessions WHERE token = $1")
        .bind(token)
        .fetch_one(db_pool)
        .await?;

    let uuid: String = session_row.try_get("uuid")?;
    let user_uuid: String = session_row.try_get("user_uuid")?;
    let expiry: NaiveDateTime = session_row.try_get("expiry")?;

    Ok((uuid, user_uuid, expiry))
}

async fn get_session_details(
    token: &str,
    db_pool: &PgPool) -> Result<(String, String, NaiveDateTime), Error>
{
    match get_session_details_sqlx(&token, &db_pool).await {
        Ok(value) => Ok(value),
        Err(error) => {
            match error {
                sqlx::Error::RowNotFound => {
                    Err(ErrorUnauthorized("Session cannot be verified."))
                },
                _ => {
                    println!("{error}");
                    Err(ErrorInternalServerError("Database error."))
                }
            }
        }
    }
}

fn get_request_user_uuid(path: Path<Url>) -> Result<String, Error> {
    match path.get("user_uuid") {
        Some(value) => Ok(value.to_string()),
        None => Err(ErrorInternalServerError("Dynamic segment named 'user_uuid' is not found"))
    }
}

fn get_auth_cookie_value(cookie_opt: Option<Cookie>) -> Result<String, Error> {
    match cookie_opt {
        Some(cookie) => Ok(cookie.value().to_string()),
        None => Err(ErrorUnauthorized("Cookie named 'auth' is not found."))
    }
}

impl FromRequest for Session {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let db_pool = req.app_data::<web::Data<PgPool>>().unwrap().clone();
        let auth_cookie = req.cookie("auth");
        let path = req.match_info().clone();

        Box::pin(async move {
            let token = get_auth_cookie_value(auth_cookie)?;
            let request_user_uuid = get_request_user_uuid(path)?;

            let (session_uuid, auth_user_uuid, auth_expiry) 
                = get_session_details(&token, &db_pool).await?;

            if request_user_uuid != auth_user_uuid {
                return Err(ErrorUnauthorized("Session is not authenticated for this user."));
            }

            if auth_expiry.timestamp() < Utc::now().naive_utc().timestamp() {
                return Err(ErrorUnauthorized("Session has expired."));
            }

            Ok(Session {
                uuid: session_uuid,
                user_uuid: auth_user_uuid,
                token: token,
            })
        })
    }
}
