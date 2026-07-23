mod authorized_person_router;
use axum::{Router, routing::get};

use crate::{
    db::DatabaseClient, handlers::authorized_handlers::logout_handler::logout,
    router::api_routes::authorized_routes::authorized_person_router::authorized_person_router,
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
