
use axum::{body::{Body, to_bytes}, extract::Request, http::StatusCode};
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
