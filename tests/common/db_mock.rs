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

    // Optional helper to pre-populate mock data for your tests
    pub async fn seed_user(&self, user: UserMock) {
        // let mut users = self.users.lock().await;
        // users.push(user);
        let _ = self
            .save_user(user.username, user.email, user.password)
            .await;
    }

    pub async fn seed_some_user(&self) {
        let payload1 = UserMock {
            username: "1testuser".to_string(),
            email: "1test@example.com".to_string(),
            password: "1SecurePassword123!".to_string(),
        };

        let payload2 = UserMock {
            username: "2testuser".to_string(),
            email: "2test@example.com".to_string(),
            password: "2SecurePassword123!".to_string(),
        };

        let payload3 = UserMock {
            username: "3testuser".to_string(),
            email: "3test@example.com".to_string(),
            password: "3SecurePassword123!".to_string(),
        };

        let users: [UserMock; 3] = [payload1, payload2, payload3];

        for user_data in users {
            self.seed_user(user_data).await;
        }
    }
}

impl DatabaseClient for DBClientMock {}
