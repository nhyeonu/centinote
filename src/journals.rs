use serde::{Serialize, Deserialize};
use actix_web::{get, post, patch, web, Responder, HttpRequest, HttpResponse};
use sqlx::{PgPool, Row, types::chrono::{DateTime, TimeZone, FixedOffset, NaiveDateTime, Utc}};
use uuid::Uuid;
use crate::State;
use crate::utils;

#[derive(Serialize)]
struct JournalList {
    uuids: Vec<String>,
}

#[derive(Deserialize)]
struct JournalCreate {
    timezone_offset: i32,
    title: String,
    body: String,
}

#[derive(Deserialize)]
struct JournalUpdate {
    title: String,
    body: String,
}

#[derive(Serialize)]
struct JournalData {
    created: String,
    title: String,
    body: String,
}

async fn query_journal_uuids_for_user(
    db_pool: &PgPool,
    user_uuid: String) -> Result<Vec<String>, sqlx::Error>
{
    let query_result = 
        sqlx::query("SELECT uuid FROM journals WHERE user_uuid = $1 ORDER BY created DESC")
        .bind(&user_uuid)
        .fetch_all(db_pool)
        .await;

    let rows = match query_result {
        Ok(rows) => rows,
        Err(error) => return Err(error)
    };

    let uuids = {
        let uuid_results: Vec<Result<String, sqlx::Error>> 
            = rows.iter().map(|row| row.try_get("uuid")).collect();

        let uuids_result: Result<Vec<String>, sqlx::Error> 
            = uuid_results.into_iter().collect();

        let uuids: Vec<String> = match uuids_result {
            Ok(vector) => vector,
            Err(error) => return Err(error)
        };

        uuids
    };

    Ok(uuids)
}

async fn create_journal_entry(
    db_pool: &PgPool,
    user_uuid: &str,
    timezone_offset: i32,
    title: &str,
    body: &str) -> Result<String, sqlx::Error>
{
    let entry_uuid = Uuid::new_v4().to_string();
    let current_timestamp = Utc::now().naive_utc();
    let insert_result = sqlx::query("INSERT INTO journals VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(&entry_uuid)
        .bind(&user_uuid)
        .bind(&current_timestamp)
        .bind(&timezone_offset)
        .bind(title)
        .bind(body)
        .execute(db_pool)
        .await;

    match insert_result {
        Ok(_) => return Ok(format!("/api/users/{}/journals/{}", user_uuid, entry_uuid)),
        Err(error) => return Err(error)
    }
}

async fn query_journal_entry(
    db_pool: &PgPool,
    user_uuid: &str,
    entry_uuid: &str) -> Result<JournalData, sqlx::Error>
{
    let journal_entry_query_result = 
        sqlx::query("SELECT * FROM journals WHERE uuid = $1 AND user_uuid = $2")
        .bind(entry_uuid)
        .bind(user_uuid)
        .fetch_one(db_pool)
        .await;

    let journal_entry_row = journal_entry_query_result?;

    let created: DateTime<FixedOffset> = {
        let created_utc_naive: NaiveDateTime = journal_entry_row.try_get("created")?;
        let timezone_offset_minute: i32 = journal_entry_row.try_get("timezone_offset")?;

        let timezone_offset = match FixedOffset::west_opt(timezone_offset_minute * 60) {
            Some(value) => value,
            None => {
                println!("Out of bound timezone offset!");
                FixedOffset::west_opt(0).unwrap()
            }
        };

        timezone_offset.from_utc_datetime(&created_utc_naive)
    };

    let title: String = journal_entry_row.try_get("title")?;
    let body: String = journal_entry_row.try_get("body")?;

    Ok(JournalData {
        created: created.to_rfc3339(),
        title: title,
        body: body
    })
}

#[get("/users/{user_uuid}/journals")]
async fn get_list(
    data: web::Data<State<'_>>,
    path: web::Path<String>,
    req: HttpRequest) -> impl Responder
{
    let user_uuid = {
        let request_user_uuid = path.into_inner();
        let trusted_user_uuid = utils::verify_request_token!(&data.db_pool, &req);

        if request_user_uuid != trusted_user_uuid {
            return HttpResponse::Unauthorized().finish();
        }

        request_user_uuid
    };

    let uuids = match query_journal_uuids_for_user(&data.db_pool, user_uuid).await {
        Ok(uuids) => uuids,
        Err(error) => {
            match error {
                sqlx::Error::RowNotFound => {
                    return web::Json(JournalList {
                        uuids: Vec::new(),
                    }).respond_to(&req).map_into_boxed_body()
                },
                _ => {
                    println!("{}", error);
                    return HttpResponse::InternalServerError().finish();
                }
            }
        }
    };

    web::Json(JournalList {
        uuids: uuids,
    }).respond_to(&req).map_into_boxed_body()
}

#[post("/users/{user_uuid}/journals")]
async fn post(
    data: web::Data<State<'_>>,
    req: HttpRequest,
    info: web::Json<JournalCreate>,
    path: web::Path<String>) -> impl Responder 
{
    if info.title.len() > 128 || info.body.len() > 2048 {
        return HttpResponse::BadRequest().finish();
    }

    if info.timezone_offset < -720 || info.timezone_offset > 840 {
        return HttpResponse::BadRequest().finish();
    }

    let user_uuid = {
        let request_user_uuid = path.into_inner();
        let trusted_user_uuid = utils::verify_request_token!(&data.db_pool, &req);

        if request_user_uuid != trusted_user_uuid {
            return HttpResponse::Unauthorized().finish();
        }

        request_user_uuid
    };

    let create_result = create_journal_entry(
        &data.db_pool,
        &user_uuid,
        info.timezone_offset,
        &info.title,
        &info.body
    ).await;
    
    match create_result {
        Ok(entry_path) => { 
            let response = HttpResponse::Created()
                .insert_header(("Location", entry_path))
                .finish();
            return response;
        },
        Err(error) => {
            println!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
    }
}

#[get("/users/{user_uuid}/journals/{entry_uuid}")]
async fn get(
    data: web::Data<State<'_>>,
    req: HttpRequest,
    path: web::Path<(String, String)>) -> impl Responder 
{
    let trusted_user_uuid = utils::verify_request_token!(&data.db_pool, &req);
    let (request_user_uuid, journal_entry_uuid) = path.into_inner();

    if request_user_uuid != trusted_user_uuid {
        return HttpResponse::Unauthorized().finish();
    }

    let journal_data_result = 
        query_journal_entry(&data.db_pool, &request_user_uuid, &journal_entry_uuid).await;

    let journal_data = match journal_data_result {
        Ok(data) => data,
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

    web::Json(journal_data).respond_to(&req).map_into_boxed_body()
}

#[patch("/users/{user_uuid}/journals/{entry_uuid}")]
async fn patch(
    data: web::Data<State<'_>>,
    req: HttpRequest,
    info: web::Json<JournalUpdate>,
    path: web::Path<(String, String)>) -> impl Responder 
{
    let trusted_user_uuid = utils::verify_request_token!(&data.db_pool, &req);
    let (request_user_uuid, entry_uuid) = path.into_inner();

    if request_user_uuid != trusted_user_uuid {
        return HttpResponse::Unauthorized().finish();
    }

    let select_result = 
        sqlx::query("SELECT COUNT(*) AS count FROM journals WHERE uuid = $1 AND user_uuid = $2")
        .bind(&entry_uuid)
        .bind(&trusted_user_uuid)
        .fetch_one(&data.db_pool)
        .await;

    let count_row = match select_result {
        Ok(row) => row,
        Err(error) => {
            println!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let count: i64 = match count_row.try_get("count") {
        Ok(value) => value,
        Err(error) => {
            println!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    if count == 0 {
        return HttpResponse::NotFound().finish();
    }

    let update_result = 
        sqlx::query("UPDATE journals SET title = $1, body = $2 WHERE uuid = $3 AND user_uuid = $4")
        .bind(&info.title)
        .bind(&info.body)
        .bind(&entry_uuid)
        .bind(&trusted_user_uuid)
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
