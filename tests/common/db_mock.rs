use std::sync::Arc;

use axum_auth_v2::{database::users_db::UserExt, db::DatabaseClient, models::user_model::User};
use tokio::sync::Mutex;

pub struct UserMock {
    username: String,
    email: String,
    password: String,
}

#[derive(Debug, Clone)]
pub struct DBClientMock {
    pub users: Arc<Mutex<Vec<User>>>,
}

impl DBClientMock {
    pub fn mock() -> Self {
        let users = Arc::new(Mutex::new(Vec::new()));
        DBClientMock { users }
    }
}

impl DatabaseClient for DBClientMock {}
