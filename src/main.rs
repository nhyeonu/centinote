use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

#[get("/")]
async fn root() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/auth")]
async fn auth_page() -> impl Responder {
    let body = r#"
<form method="post">
    <label for="username">Username: </label>
    <input type="text" id="username" name="username" placeholder="Username">
    <label for="password">Password: </label>
    <input type="password" id="password" name="password" placeholder="Password">
    <input type="submit" value="Login">
</form>
"#;
    HttpResponse::Ok()
        .body(body)
}

#[derive(Deserialize)]
struct Login {
    username: String,
    password: String,
}

#[post("/auth")]
async fn auth_api(form: web::Form<Login>) -> impl Responder {
    println!("Username: {} Password: {}", form.username, form.password);
    HttpResponse::Found().insert_header(("Location", "/")).finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(root)
            .service(auth_page)
            .service(auth_api)
    }).bind(("127.0.0.1", 8080))?.run().await
}
