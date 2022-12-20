use argon2::Argon2;
use sqlx::PgPool;

pub struct State<'a> {
    pub db_pool: PgPool,
    pub argon2: Argon2<'a>,
}
