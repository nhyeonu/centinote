pub mod create;
pub mod fetch;
pub mod verify_password;

pub struct User {
    pub uuid: String,
    pub username: String,
    pub password_hash: String,
}
