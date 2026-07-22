mod auth_routes;
mod authorized_person_router;

use axum::{
    Extension, Router, middleware,
    routing::{any, get},
};

use crate::{
    AllStates, db::DatabaseClient, handlers::{auth_handlers::logout, home_path_handlers::home, wrong_path_handlers::wrong_path}, middlewares::auth_middleware::auth, router::{
        auth_routes::auth_router, authorized_person_router::authorized_person_router,
    },
};

pub fn authorized_routes<T>() -> axum::Router
where
    T: DatabaseClient + Clone + 'static,
{
    let authorized_person_api = authorized_person_router();

    let router = Router::new();
    let logout_api = router.route("/logout", get(logout::<T>));

    authorized_person_api.merge(logout_api)
}

pub fn create_routes<T>(all_state: AllStates<T>) -> axum::Router
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

    let home_route = Router::new().route("/", get(home));
    // let wrong_route = Router::new()
    let app_api = Router::new()
        .merge(home_route)
        .nest("/api", api_route)
        .route("/{*wildcard}", any(wrong_path));
    app_api
}
