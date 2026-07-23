
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub async fn authorized_persons_only() -> Response {
    (StatusCode::OK, "authorized Persons only").into_response()
}