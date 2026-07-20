use std::sync::Arc;

use axum_auth_v2::{db::DatabaseClient, models::user_model::User};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct DBClientMock {
    pub users: Arc<Mutex<Vec<User>>>,
}

impl DBClientMock {
    pub fn mock() -> Self {
        let users = Arc::new(Mutex::new(Vec::new()));
        DBClientMock { users }
    }

    // Optional helper to pre-populate mock data for your tests
    pub async fn seed_user(&self, user: User) {
        let mut users = self.users.lock().await;
        users.push(user);
    }
}


impl DatabaseClient for DBClientMock {}