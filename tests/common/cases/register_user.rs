use axum::{
    body::{Body, to_bytes},
    extract::Request,
    http::StatusCode,
};
use tower::ServiceExt;

use crate::common::utils_mock::get_app_mock;

#[tokio::test]
async fn test_register_user_success() {
    let app = get_app_mock().await;

    // Build JSON payload for registration matching RegisterUserDto
    let payload = serde_json::json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "SecurePassword123!",
        "passwordConfirm": "SecurePassword123!"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Expecting 201 Created based on your register handler
    assert_eq!(response.status(), StatusCode::CREATED);

    let body_bytes = to_bytes(response.into_body(), 1024).await.unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Verify success message format
    assert!(body_string.contains("Registration successful"));
}

#[tokio::test]
async fn test_register_user_missing_fields() {
    let app = get_app_mock().await;

    // Missing 'password' and 'passwordConfirm' fields
    let payload = serde_json::json!({
        "username": "incompleteuser",
        "email": "incomplete@example.com"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Expecting 400 Bad Request due to validation or deserialization failure
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_register_user_invalid_input() {
    let app = get_app_mock().await;

    // Invalid email format and passwords do not match
    let payload = serde_json::json!({
        "username": "baduser",
        "email": "not-an-email",
        "password": "Short1!",
        "passwordConfirm": "Different1!"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Expecting 400 Bad Request because validator caught structural/field rules
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_register_user_already_exists() {
    let app = get_app_mock().await;

    let payload = serde_json::json!({
        "username": "duplicateuser",
        "email": "duplicate@example.com",
        "password": "SecurePassword123!",
        "passwordConfirm": "SecurePassword123!"
    });

    // 1. First registration should succeed
    let response_first = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response_first.status(), StatusCode::CREATED);

    // 2. Second registration with the same email/username should fail
    let response_second = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    println!("error  aaaaa: {:?}",response_second.status());

    // Expecting 409 Conflict (or your custom unique constraint error status code, usually 409/400)
    // Adjust assertion if your HttpError maps unique violations to a specific status like 409
    assert_ne!(response_second.status(), StatusCode::CREATED);
    assert_ne!(response_second.status(), StatusCode::OK);

    let body_bytes = to_bytes(response_second.into_body(), 1024).await.unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Check that it contains your registered error message for duplicate details
    assert!(
        body_string.contains("email")
            || body_string.contains("username")
            || body_string.contains("exist")
    );
}
