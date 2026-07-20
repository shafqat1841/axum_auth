use axum_auth_v2::db::DatabaseClient;

#[derive(Debug, Clone)]
pub struct DBClientMock;

impl DBClientMock {
    pub fn mock() -> Self {
        DBClientMock
    }
}


impl DatabaseClient for DBClientMock {}