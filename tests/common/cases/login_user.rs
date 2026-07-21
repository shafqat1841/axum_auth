use axum::{
    body::{Body, to_bytes},
    extract::Request,
    http::StatusCode,
};
use tower::ServiceExt;

use crate::common::utils_mock::get_app_mock;

#[tokio::test]
async fn test_login_user_success() {
    // Note: For login to succeed, the user must first exist in your mock database.
    // Ensure your get_app_mock or a helper seeds a test user beforehand,
    // or register them right before testing login.
    let app = get_app_mock().await;

    // 1. Register the user first
    let register_payload = serde_json::json!({
        "username": "loginuser",
        "email": "login@example.com",
        "password": "SecurePassword123!",
        "passwordConfirm": "SecurePassword123!"
    });

    // Register call (ignoring response since we tested it above)
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&register_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // 2. Now attempt to login with the registered credentials
    let login_payload = serde_json::json!({
        "identifier": "login@example.com",
        "password": "SecurePassword123!"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&login_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify that JWT cookies or response tokens were generated
    let headers = response.headers();
    assert!(headers.contains_key("set-cookie"));

    let body_bytes = to_bytes(response.into_body(), 2048).await.unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Verify response contains the token structure
    assert!(body_string.contains("success"));
    assert!(body_string.contains("token"));
}

#[tokio::test]
async fn test_login_user_success_with_email() {
    let app = get_app_mock().await;

    // 1. Register a user first so they exist in the mock database
    let register_payload = serde_json::json!({
        "username": "logintest",
        "email": "logintest@example.com",
        "password": "SecurePassword123!",
        "passwordConfirm": "SecurePassword123!"
    });

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&register_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // 2. Attempt login using email as identifier
    let login_payload = serde_json::json!({
        "identifier": "logintest@example.com",
        "password": "SecurePassword123!"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&login_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify set-cookie headers and body contents
    let headers = response.headers();
    assert!(headers.contains_key("set-cookie"));

    let body_bytes = to_bytes(response.into_body(), 2048).await.unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert!(body_string.contains("success"));
    assert!(body_string.contains("token"));
}

#[tokio::test]
async fn test_login_user_success_with_username() {
    let app = get_app_mock().await;

    // 1. Register a user first
    let register_payload = serde_json::json!({
        "username": "uniquename",
        "email": "uniquename@example.com",
        "password": "SecurePassword123!",
        "passwordConfirm": "SecurePassword123!"
    });

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&register_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // 2. Attempt login using username as identifier (since your logic checks both email and username)
    let login_payload = serde_json::json!({
        "identifier": "uniquename",
        "password": "SecurePassword123!"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&login_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_login_user_wrong_password() {
    let app = get_app_mock().await;

    // 1. Register user
    let register_payload = serde_json::json!({
        "username": "pwtest",
        "email": "pwtest@example.com",
        "password": "CorrectPassword123!",
        "passwordConfirm": "CorrectPassword123!"
    });

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&register_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // 2. Attempt login with wrong password
    let login_payload = serde_json::json!({
        "identifier": "pwtest@example.com",
        "password": "WrongPassword999!"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&login_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Expecting 400 Bad Request due to WrongCredentials error mapped in login handler
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body_bytes = to_bytes(response.into_body(), 1024).await.unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert!(
        body_string.contains("WrongCredentials")
            || body_string.contains("credentials")
            || body_string.contains("Invalid")
    );
}

#[tokio::test]
async fn test_login_user_not_found() {
    let app = get_app_mock().await;

    // Attempt login with a user that was never registered
    let login_payload = serde_json::json!({
        "identifier": "nonexistent@example.com",
        "password": "SomePassword123!"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&login_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Expecting 400 Bad Request because the user resolution returns None (mapped to WrongCredentials)
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_login_user_missing_fields() {
    let app = get_app_mock().await;

    // Missing 'password' field
    let login_payload = serde_json::json!({
        "identifier": "someuser@example.com"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&login_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Expecting 400 Bad Request for incomplete payloads
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
