use axum::{body::{Body, to_bytes}, extract::Request, http::StatusCode};
use tower::ServiceExt;

use crate::common::utils_mock::get_app_mock;


#[tokio::test]
pub async fn test_home_route() {
    let app = get_app_mock().await;

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = to_bytes(response.into_body(), 1024).await.unwrap();

    // Convert bytes to a UTF-8 string
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    assert_eq!(body_string, "hello world");
}