mod journals;
mod login;
mod register;
mod state;
mod utils;

use std::path::Path;
use actix_web::{web, App, HttpServer};
use argon2::Argon2;
use sqlx::{postgres::PgPoolOptions, migrate::Migrator};
use crate::state::State;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //TODO: Look up resources at relative path from the executable.
    let migrations_dir = "/usr/local/share/centinote/sql/migrations";
    let html_dir = "/usr/local/share/centinote/html";

    println!("Connecting to the database...");
    let pool_connect_result = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:insecure@db/centinote")
        .await;

    let pool = match pool_connect_result {
        Ok(pool) => {
            println!("Successfully connected to the database"); 
            pool 
        },
        Err(error) => panic!("{}", error)
    };

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
            .service(
                web::scope("/api")
                    .service(crate::login::post)
                    .service(crate::register::post)
                    .service(crate::journals::get_list)
                    .service(crate::journals::get)
                    .service(crate::journals::post)
                    .service(crate::journals::patch)
            )
            .service(actix_files::Files::new("/", html_dir).index_file("index.html"))
    }).bind(("0.0.0.0", 8080))?.run().await
}
