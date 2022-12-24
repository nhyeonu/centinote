use serde::{Serialize, Deserialize};
use actix_web::{get, web, post, Responder, HttpRequest, HttpResponse};
use sqlx::Row;
use uuid::Uuid;
use crate::State;
use crate::utils;

#[derive(Serialize, Deserialize)]
struct Journal {
    title: String,
    body: String,
}

#[derive(Serialize)]
struct JournalList {
    uuids: Vec<String>,
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

    let query_result = sqlx::query("SELECT uuid FROM journals WHERE user_uuid = $1")
        .bind(&user_uuid)
        .fetch_all(&data.db_pool)
        .await;

    let rows = match query_result {
        Ok(rows) => rows,
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

    let uuids = {
        let uuid_results: Vec<Result<String, sqlx::Error>> 
            = rows.iter().map(|row| row.try_get("uuid")).collect();

        let uuids_result: Result<Vec<String>, sqlx::Error> 
            = uuid_results.into_iter().collect();

        let uuids: Vec<String> = match uuids_result {
            Ok(vector) => vector,
            Err(error) => {
                println!("{}", error);
                return HttpResponse::InternalServerError().finish();
            }
        };

        uuids
    };

    web::Json(JournalList {
        uuids: uuids,
    }).respond_to(&req).map_into_boxed_body()
}

#[post("/users/{user_uuid}/journals")]
async fn post(
    data: web::Data<State<'_>>,
    req: HttpRequest,
    info: web::Json<Journal>,
    path: web::Path<String>) -> impl Responder 
{
    if info.title.len() > 128 || info.body.len() > 2048 {
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

    let entry_uuid = Uuid::new_v4().to_string();
    let insert_result = sqlx::query("INSERT INTO journals VALUES ($1, $2, $3, $4)")
        .bind(&entry_uuid)
        .bind(&user_uuid)
        .bind(&info.title)
        .bind(&info.body)
        .execute(&data.db_pool)
        .await;

    let entry_path = format!("/api/users/{}/journals/{}", user_uuid, entry_uuid);

    match insert_result {
        Ok(_) => { 
            let response = HttpResponse::Found()
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

    let journal_entry_query_result = 
        sqlx::query("SELECT title, body FROM journals WHERE uuid = $1 AND user_uuid = $2")
        .bind(journal_entry_uuid)
        .bind(request_user_uuid)
        .fetch_one(&data.db_pool)
        .await;

    let journal_entry_row = match journal_entry_query_result {
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

    let journal_title: String = match journal_entry_row.try_get("title") {
        Ok(title) => title,
        Err(error) => {
            println!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let journal_body: String = match journal_entry_row.try_get("body") {
        Ok(body) => body,
        Err(error) => {
            println!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    web::Json(Journal {
        title: journal_title,
        body: journal_body
    }).respond_to(&req).map_into_boxed_body()
}
