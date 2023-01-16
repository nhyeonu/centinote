pub mod from_request;
pub mod create;
pub mod delete;
pub mod refresh;

pub struct Session {
    pub uuid: String,
    pub token: String,
    pub user_uuid: String,
}
