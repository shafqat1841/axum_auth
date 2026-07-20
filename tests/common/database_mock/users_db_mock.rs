use async_trait::async_trait;
use axum_auth_v2::{database::users_db::UserExt, models::user_model::User};
use uuid::Uuid;

use crate::common::db_mock::DBClientMock;

#[async_trait]
impl UserExt for DBClientMock {
    async fn get_user(
        &self,
        user_id: Option<Uuid>,
        username: Option<&str>,
        email: Option<&str>,
    ) -> Result<Option<User>, sqlx::Error> {
        let users = self.users.lock().await;

        // Search through our in-memory vector matching the optional parameters
        let found = users.iter().find(|u| {
            let matches_id = user_id.map_or(true, |id| u.id == id);
            let matches_username = username.map_or(true, |uname| u.username == uname);
            let matches_email = email.map_or(true, |mail| u.email == mail);

            matches_id && matches_username && matches_email
        });

        Ok(found.cloned())
    }

    async fn save_user<T: Into<String> + Send>(
        &self,
        username: T,
        email: T,
        password_hash: T,
    ) -> Result<User, sqlx::Error> {
        let mut users = self.users.lock().await;

        let now = Some(chrono::Utc::now().naive_utc()); // Convert DateTime<Utc> to NaiveDateTime

        let new_user = User {
            id: Uuid::new_v4(),
            username: username.into(),
            email: email.into(),
            password_hash: password_hash.into(),
            created_at: now, // Adjust based on your User model field types
            updated_at: now,
        };

        users.push(new_user.clone());
        Ok(new_user)
    }
}
