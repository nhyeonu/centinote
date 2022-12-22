use serde::{Serialize, Deserialize};
use actix_web::{web, get, post, Responder, HttpRequest, HttpResponse};
use uuid::Uuid;
use crate::State;
use crate::auth::utils;

#[derive(Serialize, Deserialize)]
struct Journal {
    title: String,
    body: String,
}

#[post("/journals")]
async fn post(data: web::Data<State<'_>>, req: HttpRequest, info: web::Json<Journal>) -> impl Responder {
    if info.title.len() > 128 || info.body.len() > 2048 {
        return HttpResponse::BadRequest().finish();
    }

    let user_uuid = utils::verify_request!(&data.db_pool, &req);
    let entry_uuid = Uuid::new_v4().to_string();

    let insert_result = sqlx::query("INSERT INTO journals VALUES ($1, $2, $3, $4)")
        .bind(entry_uuid.clone())
        .bind(user_uuid)
        .bind(info.title.clone())
        .bind(info.body.clone())
        .execute(&data.db_pool)
        .await;

    match insert_result {
        Ok(_) => return HttpResponse::Found().insert_header(("Location", format!("/api/journals/{}", entry_uuid))).finish(),
        Err(error) => {
            println!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
    }
}

pub mod entries {
    use actix_web::{get, web, Responder, HttpRequest, HttpResponse};
    use sqlx::Row;
    use crate::State;
    use crate::auth::utils;
    use crate::app::journals::Journal;

    #[get("/journals/{entry_uuid}")]
    async fn get(data: web::Data<State<'_>>, req: HttpRequest, path: web::Path<String>) -> impl Responder {
        let journal_entry_uuid = path.into_inner();
        let journal_entry_query_result = sqlx::query("SELECT UserUuid AS user_uuid, Title AS title, Body AS body FROM journals WHERE Uuid = $1")
            .bind(journal_entry_uuid)
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

        let trusted_user_uuid = utils::verify_request!(&data.db_pool, &req);
        let journal_user_uuid: String = match journal_entry_row.try_get("user_uuid") {
            Ok(uuid) => uuid,
            Err(error) => {
                println!("{}", error);
                return HttpResponse::InternalServerError().finish();
            }
        };

        if trusted_user_uuid != journal_user_uuid {
            return HttpResponse::Unauthorized().finish();
        }

        let journal_title: String = match journal_entry_row.try_get("title") {
            Ok(title) => title,
            Err(error) => {
                println!("{}", error);
                return HttpResponse::InternalServerError().finish();
            }
        };

        let journal_body: String = match journal_entry_row.try_get("title") {
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
}
