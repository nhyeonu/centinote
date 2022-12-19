use std::path::Path;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use actix_web::cookie::{Cookie, SameSite, Expiration};
use serde::Deserialize;
use sqlx::{PgPool, Row};
use sqlx::postgres::PgPoolOptions;
use sqlx::migrate::Migrator;
use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use uuid::Uuid;

#[derive(Deserialize)]
struct Login {
    username: String,
    password: String,
}

#[post("/api/login")]
async fn login(data: web::Data<State<'_>>, info: web::Json<Login>) -> impl Responder {
    let password_select_result = sqlx::query("SELECT HashedPassword AS password_hash, Uuid AS uuid FROM users WHERE Username = $1")
        .bind(info.username.clone())
        .fetch_one(&data.db_pool)
        .await;

    let user_row = match password_select_result {
        Ok(row) => row,
        Err(error) => {
            match error {
                sqlx::Error::RowNotFound => {
                    return HttpResponse::Unauthorized().finish(); 
                },
                _ => {
                    return HttpResponse::InternalServerError().finish(); 
                }
            }
        }
    };

    let password_hash: String = match user_row.try_get("password_hash") {
        Ok(hash) => hash,
        Err(error) => {
            println!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let parsed_hash = match PasswordHash::new(&password_hash) {
        Ok(parsed) => parsed,
        Err(error) => {
            println!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    //TODO: Separate out content of this if statement to a function.
    if data.argon2.verify_password(info.password.as_bytes(), &parsed_hash).is_ok() {
        let user_uuid: String = match user_row.try_get("uuid") {
            Ok(uuid) => uuid,
            Err(error) => {
                println!("{}", error);
                return HttpResponse::InternalServerError().finish();
            }
        };

        let token: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect();

        let insert_result = sqlx::query("INSERT INTO sessions VALUES ($1, $2);")
            .bind(user_uuid)
            .bind(token.clone())
            .execute(&data.db_pool)
            .await;

        match insert_result {
            Ok(_) => {
                let cookie = Cookie::build("auth", token)
                    .http_only(true)
                    .same_site(SameSite::Strict)
                    .finish();

                return HttpResponse::Found().cookie(cookie).finish();
            },
            Err(error) => {
                println!("{}", error);
                return HttpResponse::InternalServerError().finish();
            }
        }
    } else {
        return HttpResponse::Unauthorized().finish(); 
    }
}

#[derive(Deserialize)]
struct Register {
    username: String,
    password: String,
}

#[post("/api/register")]
async fn register(data: web::Data<State<'_>>, info: web::Json<Register>) -> impl Responder {
    // len() returns number of bytes in a string and VARCHAR in SQL also counts length in bytes.
    // Username length limit must be equal to the length limit defined by scripts at sql/migrations
    if info.username.len() > 64 || info.password.len() > 64 {
        return HttpResponse::BadRequest().finish();
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = match data.argon2.hash_password(info.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(error) => {
            println!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let user_count_result = sqlx::query("SELECT COUNT(*) AS count FROM users WHERE Username = $1")
        .bind(info.username.clone())
        .fetch_one(&data.db_pool)
        .await;

    let user_count_row = match user_count_result {
        Ok(row) => row,
        Err(error) => {
            println!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let user_count: i64 = match user_count_row.try_get("count") {
        Ok(count) => count,
        Err(error) => {
            println!("{}", error);
            return HttpResponse::InternalServerError().finish();
        }
    };

    if user_count == 0 {
        let insert_request = sqlx::query("INSERT INTO users VALUES ($1, $2, $3);")
            .bind(Uuid::new_v4().to_string())
            .bind(info.username.clone())
            .bind(password_hash)
            .execute(&data.db_pool)
            .await;

        match insert_request {
            Ok(_) => return HttpResponse::Created().finish(),
            Err(error) => {
                println!("{}", error);
                return HttpResponse::InternalServerError().finish();
            }
        }
        
    } else {
        return HttpResponse::Conflict().finish();
    }
}

struct State<'a> {
    db_pool: PgPool,
    argon2: Argon2<'a>,
}

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
            .service(login)
            .service(register)
            .service(actix_files::Files::new("/", "/usr/local/share/centinote/html").index_file("index.html"))
    }).bind(("0.0.0.0", 8080))?.run().await
}
