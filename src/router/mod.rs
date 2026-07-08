mod auth_routes;
mod authorized_person_router;

use std::sync::Arc;

use axum::{Extension, Router, middleware, routing::get};

use crate::{
    AppState,
    middlewares::auth_middleware::auth,
    router::{auth_routes::auth_router, authorized_person_router::authorized_person_router},
};

pub fn authorized_routes() -> axum::Router {
    let authorized_person_api = authorized_person_router().layer(middleware::from_fn(auth));
    authorized_person_api
}

pub fn create_routes(app_state: Arc<AppState>) -> axum::Router {
    let router = Router::new();
    let auth_api = auth_router();
    let authorized_api = authorized_routes();
    let api_route = router
        .nest("/auth", auth_api)
        .merge(authorized_api)
        .layer(Extension(app_state));

    let home_route = Router::new().route("/", get(home));

    let app_api = Router::new().merge(home_route).nest("/api", api_route);
    app_api
}

async fn home() -> &'static str {
    "hello world"
}
