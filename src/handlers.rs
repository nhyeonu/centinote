use actix_web::{
    get, post, patch, delete, web, HttpRequest, HttpResponse, Responder, Error, 
    cookie::{Cookie, SameSite},
    error::ErrorUnauthorized
};
use serde::{Serialize, Deserialize};
use sqlx::PgPool;

use crate::entry::Entry;
use crate::session::Session;
use crate::user::User;

/*
===== POST /api/users =====

This handler creates a user on success.

Request JSON example: { "username": "myusername", "password": "mypassword" }

Notable HTTP status codes:
 409 Conflict: Username is already taken.
*/

#[derive(Deserialize)]
struct UserCreate {
    username: String,
    password: String,
}

#[post("/api/users")]
async fn user_create(
    db_pool: web::Data<PgPool>,
    info: web::Json<UserCreate>) -> Result<HttpResponse, Error> 
{
    let user = User::create(&db_pool, &info.username, &info.password).await?;

    let user_path = format!("/api/users/{}", &user.uuid);
    Ok(HttpResponse::Created().insert_header(("Location", user_path)).finish())
}

/* 
===== POST /api/login =====

Request JSON example: { "username": "myusername", "password": "mypassword" }

Three cookies are sent on a successful login attempt.
 auth: Authentication token for this session. HTTP only.
 session_uuid: Contains UUID for the session just created.
 user_uuid: Contains UUID of the user.

Notable HTTP status codes:
 401 Unauthorized: Username and/or password is wrong.
*/

#[derive(Deserialize)]
struct Login {
    username: String,
    password: String,
}

fn build_cookie<'a>(
    name: &'a str,
    value: &'a str,
    http_only: bool) -> Cookie<'a> 
{
    Cookie::build(name, value).same_site(SameSite::Strict).path("/").http_only(http_only).finish()
}

#[post("/api/login")]
async fn login(
    db_pool: web::Data<PgPool>,
    info: web::Json<Login>) -> Result<HttpResponse, Error> 
{
    let user = match User::by_username(&db_pool, &info.username).await {
        Ok(value) => value,
        Err(error) => {
            if error.as_response_error().status_code().as_u16() == 404 {
                return Err(ErrorUnauthorized("User not found."));
            } else {
                return Err(error);
            }
        }
    };
    user.verify_password(&info.password)?;
    
    let session = Session::create(&db_pool, &user.uuid).await?;

    let session_uuid = build_cookie("session_uuid", &session.uuid, false);
    let user_uuid = build_cookie("user_uuid", &session.user_uuid, false);
    let auth = build_cookie("auth", &session.token, true);

    let session_path = format!("/api/users/{}/sessions/{}", session.user_uuid, session.uuid);
    let response = HttpResponse::Created()
       .insert_header(("Location", session_path))
       .cookie(session_uuid)
       .cookie(user_uuid)
       .cookie(auth)
       .finish();

    Ok(response)
}

/*
===== POST /api/users/{user_uuid}/sessions/{session_uuid} =====

Refresh the specified session's lifetime. 
Session's expiry timestamp will be 30 minutes from now on success.
A session is only authorized for it's own refresh.

Notable HTTP status codes:
 401 Unauthorized: Session is not authorized for refreshing the specified session.
*/

#[post("/api/users/{user_uuid}/sessions/{session_uuid}")]
async fn session_refresh(
    session: Session,
    db_pool: web::Data<PgPool>,
    path: web::Path<(String, String)>) -> Result<HttpResponse, Error>
{
    let (_, target_session_uuid) = path.into_inner();

    if session.uuid == target_session_uuid {
        session.refresh(&db_pool).await?;
        Ok(HttpResponse::Ok().finish())
    } else {
        Err(ErrorUnauthorized("Session UUIDs does not match."))
    }
}

/*
===== DELETE /api/users/{user_uuid}/sessions/{session_uuid} =====

Delete the specified session on success. Useful for logging out.

Notable HTTP status codes:
 401 Unauthorized: Session is not authorized for deletion of specified session.
*/

#[delete("/api/users/{user_uuid}/sessions/{session_uuid}")]
async fn session_delete(
    session: Session,
    db_pool: web::Data<PgPool>) -> Result<HttpResponse, Error>
{
    session.delete(&db_pool).await?;
    Ok(HttpResponse::Ok().finish())
}

/*
===== GET /api/users/{user_uuid}/entries =====

This handler on success, responds with a list of entry UUIDs that belong to a user, recent first.

Response JSON example: 
{ "uuids": ["5315486c-02ee-4712-9793-b002193d0275", "2deffb77-b215-47b5-a074-ddd4127cc4b5"] }

Notable HTTP status codes:
 401 Unauthorized: Session is not authorized for the requested entry.
*/

#[derive(Serialize)]
struct EntryList {
    uuids: Vec<String>,
}

#[get("/api/users/{user_uuid}/entries")]
async fn entry_list(
    session: Session,
    req: HttpRequest,
    db_pool: web::Data<PgPool>) -> Result<HttpResponse, Error> 
{
    let uuids = Entry::uuids_by_user(&db_pool, &session.user_uuid).await?;

    let response = web::Json(EntryList {
        uuids: uuids,
    }).respond_to(&req).map_into_boxed_body();

    Ok(response)
}

/*
===== GET /api/users/{user_uuid}/entries/{entry_uuid} =====

This handler responds with content of an entry on success.

Response JSON example:
{
    "created": "2023-01-07T06:29:16.035754+09:00",
    "title": "My Title",
    "body": "I did nothing today."
}

The 'created' field is formated in ISO 8601.

Notable HTTP status codes:
 401 Unauthorized: Session is not authorized for the requested entry.
 404 Not Found: Requested entry does not exist.
*/

#[derive(Serialize)]
struct EntryDetail {
    created: String,
    title: String,
    body: String,
}

#[get("/api/users/{user_uuid}/entries/{entry_uuid}")]
async fn entry_detail(
    session: Session,
    req: HttpRequest,
    db_pool: web::Data<PgPool>,
    path: web::Path<(String, String)>) -> Result<HttpResponse, Error>
{
    let (_, entry_uuid) = path.into_inner();
    let entry = Entry::by_uuid_and_user(&db_pool, &entry_uuid, &session.user_uuid).await?;

    let response = web::Json(EntryDetail {
        created: entry.created,
        title: entry.title,
        body: entry.body,
    }).respond_to(&req).map_into_boxed_body();

    Ok(response)
}

/*
===== POST /api/users/{user_uuid}/entries =====

This handler creates an entry on success.

Request JSON example:
{
    "title": "My Title",
    "body": "I did nothing today.",
    "timezone_offset": -540
}

Value of the 'timezone_offset' field is defined with JS getTimezoneOffset() in mind.

Notable HTTP status codes:
 401 Unauthorized: Session is not authorized for this user.
 404 Not Found: Requested entry does not exist.
*/

#[derive(Deserialize)]
struct EntryCreate {
    timezone_offset: i32,
    title: String,
    body: String,
}

#[post("/api/users/{user_uuid}/entries")]
async fn entry_create(
    session: Session,
    db_pool: web::Data<PgPool>,
    info: web::Json<EntryCreate>) -> Result<HttpResponse, Error>
{
    let entry = Entry::create(
        &db_pool,
        info.timezone_offset,
        &session.user_uuid,
        &info.title,
        &info.body).await?;

    let entry_path = format!("/api/users/{}/entries/{}", &session.user_uuid, &entry.uuid);
    Ok(HttpResponse::Created().insert_header(("Location", entry_path)).finish())
}

/*
===== PATCH /api/users/{user_uuid}/entries/{entry_uuid} =====

This handler replaces content of an existing entry on success. 
The 'created' timestamp will not be modified.

Request JSON example:
{
    "title": "New Title",
    "body": "I did nothing yesterday."
}

Notable HTTP status codes:
 401 Unauthorized: Session is not authorized for the requested entry.
 404 Not Found: Requested entry does not exist.
*/

#[derive(Deserialize)]
struct EntryUpdate {
    title: String,
    body: String,
}

#[patch("/api/users/{user_uuid}/entries/{entry_uuid}")]
async fn entry_update(
    session: Session,
    db_pool: web::Data<PgPool>,
    info: web::Json<EntryUpdate>,
    path: web::Path<(String, String)>) -> Result<HttpResponse, Error>
{
    let (_, entry_uuid) = path.into_inner();

    let entry = Entry::by_uuid_and_user(&db_pool, &entry_uuid, &session.user_uuid).await?;
    entry.update(&db_pool, &info.title, &info.body).await?;

    Ok(HttpResponse::Ok().finish())
}

/*
===== DELETE /api/users/{user_uuid}/entries/{entry_uuid} =====

This handler deletes an entry on success.

Notable HTTP status codes:
 401 Unauthorized: Session is not authorized for the requested entry.
 404 Not Found: Requested entry does not exist.
*/

#[delete("/api/users/{user_uuid}/entries/{entry_uuid}")]
async fn entry_delete(
    session: Session,
    db_pool: web::Data<PgPool>,
    path: web::Path<(String, String)>) -> Result<HttpResponse, Error>
{
    let (_, entry_uuid) = path.into_inner();

    let entry = Entry::by_uuid_and_user(&db_pool, &entry_uuid, &session.user_uuid).await?;
    entry.delete(&db_pool).await?;

    Ok(HttpResponse::Ok().finish())
}
