use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};

#[get("/")]
async fn root() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/auth")]
async fn auth_page() -> impl Responder {
    let body = r#"
<form action="/method="post">
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(root)
            .service(auth_page)
    }).bind(("127.0.0.1", 8080))?.run().await
}
