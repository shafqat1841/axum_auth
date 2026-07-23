use std::sync::Arc;

use sqlx::{Pool, Postgres};
use tokio::sync::Mutex;

use crate::{database::users_db::UserExt, models::user_model::User};

pub trait DatabaseClient: Send + Sync + UserExt {}

#[derive(Debug, Clone)]
pub struct DBClient {
    pub pool: Pool<Postgres>,
}

impl DBClient {
    pub fn new(pool: Pool<Postgres>) -> Self {
        DBClient { pool }
    }
}

impl DatabaseClient for DBClient {}

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
