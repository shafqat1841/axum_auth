use axum::{body::{Body, to_bytes}, extract::Request, http::StatusCode};
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
