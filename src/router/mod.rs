mod api_routes;

use axum::{
    Router,
    routing::{any, get},
};

use crate::{
    AllStates,
    db::DatabaseClient,
    handlers::{home_path_handlers::home, wrong_path_handlers::wrong_path},
    router::api_routes::api_routes,
};

pub fn create_routes<T>(all_state: AllStates<T>) -> axum::Router
where
    T: DatabaseClient + Clone + 'static,
{
    let api_route = api_routes(all_state);

    let home_route = Router::new().route("/", get(home));
    // let wrong_route = Router::new()
    let app_api = Router::new()
        .merge(home_route)
        .nest("/api", api_route)
        .route("/{*wildcard}", any(wrong_path));
    app_api
}
