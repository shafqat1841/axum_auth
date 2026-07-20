use axum::{body::Body, extract::Request, http::StatusCode};
use tower::ServiceExt;

use crate::common::utils_mock::get_app_mock;

#[tokio::test]
pub async fn test_authorized_route_fails_without_auth() {
    let app = get_app_mock().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/logout")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
pub async fn test_auth_route_exists() {
    let app = get_app_mock().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/auth")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
