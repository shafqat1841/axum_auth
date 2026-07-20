use sqlx::{Pool, Postgres};

use crate::database::users_db::UserExt;

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
