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

#[post("/users/{user_uuid}/journals")]
async fn post(data: web::Data<State<'_>>, req: HttpRequest, info: web::Json<Journal>, path: web::Path<String>) -> impl Responder {
    if info.title.len() > 128 || info.body.len() > 2048 {
        return HttpResponse::BadRequest().finish();
    }

    let request_user_uuid = path.into_inner();
    let trusted_user_uuid = utils::verify_request_token!(&data.db_pool, &req);

    if request_user_uuid != trusted_user_uuid {
        return HttpResponse::Unauthorized().finish();
    }

    let entry_uuid = Uuid::new_v4().to_string();
    let insert_result = sqlx::query("INSERT INTO journals VALUES ($1, $2, $3, $4)")
        .bind(&entry_uuid)
        .bind(&request_user_uuid)
        .bind(&info.title)
        .bind(&info.body)
        .execute(&data.db_pool)
        .await;

    match insert_result {
        Ok(_) => return HttpResponse::Found().insert_header(("Location", format!("/api/users/{request_user_uuid}/journals/{entry_uuid}"))).finish(),
        Err(error) => {
            println!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
    }
}

#[get("/users/{user_uuid}/journals/{entry_uuid}")]
async fn get(data: web::Data<State<'_>>, req: HttpRequest, path: web::Path<(String, String)>) -> impl Responder {
    let trusted_user_uuid = utils::verify_request_token!(&data.db_pool, &req);
    let (request_user_uuid, journal_entry_uuid) = path.into_inner();

    if request_user_uuid != trusted_user_uuid {
        return HttpResponse::Unauthorized().finish();
    }

    let journal_entry_query_result = sqlx::query("SELECT user_uuid, title, body FROM journals WHERE uuid = $1 AND user_uuid = $2")
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
