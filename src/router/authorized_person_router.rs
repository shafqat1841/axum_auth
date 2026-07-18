use axum::{Router, routing::get};

use crate::handlers::authorized_handlers::authorized_persons_only;


pub fn authorized_person_router() -> Router {
    let router = Router::new();
    let api = router.route("/authorized_route", get(authorized_persons_only));
    api
}
