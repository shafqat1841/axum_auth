mod auth_routes;
mod authorized_person_router;

use axum::{Extension, Router, middleware, routing::get};

use crate::{
    AllStates,
    db::DatabaseClient,
    middlewares::auth_middleware::auth,
    router::{
        auth_routes::{auth_router, logout},
        authorized_person_router::authorized_person_router,
    },
};

pub fn authorized_routes() -> axum::Router {
    let authorized_person_api = authorized_person_router();

    let router = Router::new();
    let logout_api = router.route("/logout", get(logout));

    authorized_person_api.merge(logout_api)
}

pub fn create_routes<T>(all_state: AllStates<T>) -> axum::Router
where
    T: DatabaseClient + Clone + 'static,
{
    let router = Router::new();
    let auth_api = auth_router();

    let authorized_api = authorized_routes().layer(middleware::from_fn(auth));

    let api_route = router
        .nest("/auth", auth_api)
        .merge(authorized_api)
        .layer(Extension(all_state));

    let home_route = Router::new().route("/", get(home));

    let app_api = Router::new().merge(home_route).nest("/api", api_route);
    app_api
}

async fn home() -> &'static str {
    "hello world"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn home_fn() {
        let result = home().await;
        assert_eq!(result, "hello world");
    }
}
