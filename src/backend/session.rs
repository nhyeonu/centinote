use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use actix_web::{
    web, FromRequest, HttpRequest,
    cookie::Cookie,
    dev::{Payload, Path, Url},
    error::{Error, ErrorUnauthorized, ErrorInternalServerError, ErrorNotFound}
};
use sqlx::{PgPool, Row};
use chrono::{Duration, NaiveDateTime, Utc};
use std::pin::Pin;
use std::future::Future;
use uuid::Uuid;

pub struct Session {
    pub uuid: String,
    pub user_uuid: String,
    pub token: String,
}

fn get_auth_cookie_value(cookie_opt: Option<Cookie>) -> Result<String, Error> {
    match cookie_opt {
        Some(cookie) => Ok(cookie.value().to_string()),
        None => Err(ErrorUnauthorized("Cookie named 'auth' is not found."))
    }
}

fn get_request_user_uuid(path: Path<Url>) -> Result<String, Error> {
    match path.get("user_uuid") {
        Some(value) => Ok(value.to_string()),
        None => Err(ErrorInternalServerError("Dynamic segment named 'user_uuid' is not found"))
    }
}

async fn create_auth_token(
    db_pool: &PgPool,
    user_uuid: &str) -> Result<(String, String), sqlx::Error> 
{
    let uuid = Uuid::new_v4().to_string();
    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    sqlx::query("INSERT INTO sessions VALUES ($1, $2, $3, $4);")
        .bind(&uuid)
        .bind(user_uuid)
        .bind(Utc::now().naive_utc() + Duration::minutes(30))
        .bind(&token)
        .execute(db_pool)
        .await?;

    Ok((uuid, token))
}


impl Session {
    pub async fn create(db_pool: &PgPool, user_uuid: &str) -> Result<Self, Error> {
        match create_auth_token(db_pool, user_uuid).await {
            Ok((uuid, token)) => Ok(Session {
                uuid: uuid,
                user_uuid: user_uuid.to_string(),
                token: token
            }),
            Err(error) => {
                println!("{error}");
                Err(ErrorInternalServerError("Database error."))
            }
        }
    }
}

async fn delete_session(
    db_pool: &PgPool,
    session_uuid: &str,
    user_uuid: &str) -> Result<(), Error>
{
    let delete_result = 
        sqlx::query("DELETE FROM sessions WHERE uuid = $1 AND user_uuid = $2")
        .bind(session_uuid)
        .bind(user_uuid)
        .execute(db_pool)
        .await;

    let query_result = match delete_result {
        Ok(value) => value,
        Err(error) => {
            println!("{error}");
            return Err(ErrorInternalServerError("Database error."));
        }
    };

    if query_result.rows_affected() == 0 {
        return Err(ErrorNotFound("Entry not found."));
    }

    Ok(())
}

impl Session {
    pub async fn delete(self, db_pool: &PgPool) -> Result<Self, Error> {
        delete_session(db_pool, &self.uuid, &self.user_uuid).await?;
        Ok(self)
    }
}

async fn refresh_auth_token(
    db_pool: &PgPool,
    session_uuid: &str,
    user_uuid: &str) -> Result<(), Error>
{
    let update_result = 
        sqlx::query("UPDATE sessions SET expiry = $1 WHERE uuid = $2 AND user_uuid = $3")
        .bind(Utc::now().naive_utc() + Duration::minutes(30))
        .bind(session_uuid)
        .bind(user_uuid)
        .execute(db_pool)
        .await;

    match update_result {
        Ok(_) => Ok(()),
        Err(error) => {
            println!("{}", error);
            Err(ErrorInternalServerError("Database error."))
        }
    }
}

impl Session {
    pub async fn refresh(self, db_pool: &PgPool) -> Result<Self, Error> {
        refresh_auth_token(db_pool, &self.uuid, &self.user_uuid).await?;
        Ok(self)
    }
}

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
