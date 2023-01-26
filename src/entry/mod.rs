pub mod create;
pub mod delete;
pub mod fetch;
pub mod list;
pub mod update;
mod utils;

pub struct Entry {
    pub created: String,
    pub title: String,
    pub body: String,
    pub uuid: String,
    pub user_uuid: String,
}
