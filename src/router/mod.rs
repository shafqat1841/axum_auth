mod auth_routes;
mod authorized_person_router;

use axum::{
    Extension, Router,
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{any, get},
};

use crate::{
    AllStates,
    db::DatabaseClient,
    middlewares::auth_middleware::auth,
    router::{
        auth_routes::{auth_router, logout},
        authorized_person_router::authorized_person_router,
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

async fn wrong_path() -> Response {
    (StatusCode::NOT_FOUND, "No path like this exist").into_response()
}

async fn home() -> Response {
    (StatusCode::OK, "Home route").into_response()
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;

    use super::*;

    #[tokio::test]
    async fn home_fn_body_test() {
        let result = home().await;
        assert_eq!(result.status(), 200);

        let body_bytes = to_bytes(result.into_body(), 1024).await.unwrap();

        // Convert bytes to a UTF-8 string
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

        assert_eq!(body_string, "Home route");
    }

    #[tokio::test]
    async fn wrong_path_fn_body_test() {
        let result = wrong_path().await;
        assert_eq!(result.status(), 404);

        let body_bytes = to_bytes(result.into_body(), 1024).await.unwrap();

        // Convert bytes to a UTF-8 string
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

        assert_eq!(body_string, "No path like this exist");
    }
}
