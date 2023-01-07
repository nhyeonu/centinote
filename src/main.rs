mod journals;
mod login;
mod state;
mod utils;
mod users;

use std::env;
use std::path::Path;
use actix_web::{web, App, HttpServer};
use argon2::Argon2;
use sqlx::{PgPool, postgres::PgPoolOptions, migrate::Migrator};
use crate::state::State;

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
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(State {
                db_pool: pool.clone(),
                argon2: Argon2::default(),
            }))
            .service(crate::login::post_login)
            .service(crate::login::post_session)
            .service(crate::login::delete_session)
            .service(crate::journals::get_list)
            .service(crate::journals::get)
            .service(crate::journals::post)
            .service(crate::journals::patch)
            .service(crate::journals::delete)
            .service(crate::users::get)
            .service(crate::users::post)
            .service(actix_files::Files::new("/", html_dir).index_file("redirect.html"))
    }).bind(("0.0.0.0", 8080))?.run().await
}
