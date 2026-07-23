mod auth_routes;
mod authorized_routes;
use axum::{Extension, Router, middleware};

use crate::{
    AllStates,
    db::DatabaseClient,
    middlewares::auth_middleware::auth,
    router::api_routes::{auth_routes::auth_router, authorized_routes::authorized_routes},
};

pub fn api_routes<T>(all_state: AllStates<T>) -> Router
where
    T: DatabaseClient + Clone + 'static,
{
    let router = Router::new();
    let auth_api = auth_router::<T>();

    let authorized_api = authorized_routes::<T>().layer(middleware::from_fn(auth::<T>));

    let api_route = router
        .nest("/auth", auth_api)
        .merge(authorized_api)
        .layer(Extension(all_state));

    api_route
}
