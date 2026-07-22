use crate::{
    db::DatabaseClient,
    handlers::auth_handlers::{login, register},
};

use axum::{Router, routing::post};

pub fn auth_router<T>() -> Router
where
    T: DatabaseClient + Clone + 'static,
{
    Router::new()
        .route("/register", post(register::<T>))
        .route("/login", post(login::<T>))
}
