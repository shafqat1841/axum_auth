use sqlx::{Pool, Postgres};

pub trait DatabaseClient: Send + Sync {}

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
