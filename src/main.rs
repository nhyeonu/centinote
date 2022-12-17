use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use sqlx::{ConnectOptions, PgPool};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

#[get("/")]
async fn root() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/login")]
async fn login_page() -> impl Responder {
    let body = r#"
<form method="post">
    <label for="username">Username: </label>
    <input type="text" id="username" name="username" placeholder="Username">
    <label for="password">Password: </label>
    <input type="password" id="password" name="password" placeholder="Password">
    <input type="submit" value="Login">
</form>
"#;
    HttpResponse::Ok().body(body)
}

#[get("/register")]
async fn register_page() -> impl Responder {
    let body = r#"
<form method="post">
    <label for="username">Username: </label>
    <input type="text" id="username" name="username" placeholder="Username">
    <label for="password">Password: </label>
    <input type="password" id="password" name="password" placeholder="Password">
    <input type="submit" value="Register">
</form>
"#;
    HttpResponse::Ok().body(body)
}

#[derive(Deserialize)]
struct Login {
    username: String,
    password: String,
}

#[post("/login")]
async fn login_api(form: web::Form<Login>) -> impl Responder {
    println!("Username: {} Password: {}", form.username, form.password);
    HttpResponse::Found().insert_header(("Location", "/")).finish()
}

#[derive(Deserialize)]
struct Register {
    username: String,
    password: String,
}

#[post("/register")]
async fn register_api(form: web::Form<Login>) -> impl Responder {
    println!("Username: {} Password: {}", form.username, form.password);
    HttpResponse::Found().insert_header(("Location", "/")).finish()
}

struct State {
    db_pool: PgPool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(State {
                db_pool: PgPoolOptions::new()
                    .max_connections(5)
                    .connect_lazy("postgres://postgres:insecure@db/centinote")
                    .unwrap()
            }))
            .service(root)
            .service(login_page)
            .service(login_api)
            .service(register_page)
            .service(register_api)
    }).bind(("0.0.0.0", 8080))?.run().await
}
