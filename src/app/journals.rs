use serde::Deserialize;
use actix_web::{web, post, Responder, HttpRequest, HttpResponse};
use uuid::Uuid;
use crate::State;
use crate::auth::utils;

#[derive(Deserialize)]
struct Entry {
    title: String,
    body: String,
}

#[post("/journals")]
async fn post(data: web::Data<State<'_>>, req: HttpRequest, info: web::Json<Entry>) -> impl Responder {
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
