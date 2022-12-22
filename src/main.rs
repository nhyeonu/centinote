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
    println!("Connecting to the database...");
    let pool = match PgPoolOptions::new().max_connections(5).connect("postgres://postgres:insecure@db/centinote").await {
        Ok(pool) => {
            println!("Successfully connected to the database"); 
            pool 
        },
        Err(error) => panic!("{}", error)
    };

    //TODO: Look up sql scripts at relative path from the executable.
    let migrator = match Migrator::new(Path::new("/usr/local/share/centinote/sql/migrations")).await {
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
                    .service(crate::journals::post)
                    .service(crate::journals::get)
            )
            .service(actix_files::Files::new("/", "/usr/local/share/centinote/html").index_file("index.html"))
    }).bind(("0.0.0.0", 8080))?.run().await
}
