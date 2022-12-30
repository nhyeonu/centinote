use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::Row;
use serde::Serialize;
use crate::State;
use crate::utils;

#[derive(Serialize)]
struct UserData {
    username: String,
}

#[get("/users/{user_uuid}")]
async fn get(
    data: web::Data<State<'_>>,
    req: HttpRequest,
    path: web::Path<String>) -> impl Responder 
{
    let user_uuid = {
        let request_user_uuid = path.into_inner();
        let trusted_user_uuid = utils::verify_request_token!(&data.db_pool, &req);

        if request_user_uuid != trusted_user_uuid {
            return HttpResponse::Unauthorized().finish();
        }

        trusted_user_uuid
    };

    let select_result = 
        sqlx::query("SELECT username FROM users WHERE uuid = $1")
        .bind(&user_uuid)
        .fetch_one(&data.db_pool)
        .await;

    let user_row = match select_result {
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

    let username: String = match user_row.try_get("username") {
        Ok(uuid) => uuid,
        Err(error) => {
            println!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    web::Json(UserData {
        username: username,
    }).respond_to(&req).map_into_boxed_body()
}
