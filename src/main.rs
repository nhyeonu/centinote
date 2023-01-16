mod entry;
mod session;
mod handlers;
mod user;

use std::env;
use std::path::Path;
use actix_web::{web, App, HttpServer};
use sqlx::{PgPool, postgres::PgPoolOptions, migrate::Migrator};

async fn db_connect() -> PgPool {
    let db_host = match env::var("CENTINOTE_DB_HOST") {
        Ok(value) => value,
        Err(_) => "localhost".to_string()
    };

    let db_database = match env::var("CENTINOTE_DB_DATABASE") {
        Ok(value) => value,
        Err(_) => "centinote".to_string()
    };

    let db_username = match env::var("CENTINOTE_DB_USERNAME") {
        Ok(value) => value,
        Err(_) => "".to_string()
    };

    let db_password = match env::var("CENTINOTE_DB_PASSWORD") {
        Ok(value) => value,
        Err(_) => "".to_string()
    };

    let db_url = format!("postgres://{db_username}:{db_password}@{db_host}/{db_database}");

    println!("Connecting to the database...");
    let pool_connect_result = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await;

    let pool = match pool_connect_result {
        Ok(pool) => {
            println!("Successfully connected to the database"); 
            pool 
        },
        Err(error) => panic!("{}", error)
    };

    pool
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //TODO: Look up resources at relative path from the executable.
    let migrations_dir = "/usr/local/share/centinote/sql/migrations";
    let html_dir = "/usr/local/share/centinote/html";
    let pool = db_connect().await;

    let migrator_create_result = Migrator::new(Path::new(migrations_dir)).await;
    let migrator = match migrator_create_result {
        Ok(migrator) => migrator,
        Err(error) => panic!("{}", error)
    };

    println!("Running migration...");
    match migrator.run(&pool).await {
        Ok(_) => println!("Migration done!"),
        Err(error) => panic!("{}", error)
    }

    println!("Starting the web server...");
    HttpServer::new(move || { App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(handlers::user_create)
            .service(handlers::login)
            .service(handlers::session_refresh)
            .service(handlers::session_delete)
            .service(handlers::entry_list)
            .service(handlers::entry_detail)
            .service(handlers::entry_create)
            .service(handlers::entry_update)
            .service(handlers::entry_delete)
            .service(actix_files::Files::new("/", html_dir).index_file("redirect.html"))
    }).bind(("0.0.0.0", 8080))?.run().await
}
