use axum::{
    body::{Body, to_bytes},
    extract::Request,
    http::StatusCode,
};
use tower::ServiceExt;

use crate::common::utils_mock::get_app_mock;

#[tokio::test]
async fn test_logout_success() {
    let app = get_app_mock().await;

    // 1. Register and Login a user to generate tokens and populate the refresh token cache
    let register_payload = serde_json::json!({
        "username": "logoutuser",
        "email": "logout@example.com",
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

    let login_payload = serde_json::json!({
        "identifier": "logout@example.com",
        "password": "SecurePassword123!"
    });

    let login_response = app
        .clone()
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

    assert_eq!(login_response.status(), StatusCode::OK);

    let cookies: Vec<String> = login_response
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|val| val.to_str().unwrap().to_string())
        .collect();

    // Join them together for the cookie header
    let cookie_header = cookies.join("; ");

    // 2. Perform Logout request passing the refresh_token cookie
    let response = app
        .oneshot(
            Request::builder()
                .method("GET") // Match your route definition method (GET or POST)
                .uri("/api/logout")
                .header("cookie", cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = to_bytes(response.into_body(), 1024).await.unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert!(body_string.contains("Logout successful"));
}

#[tokio::test]
async fn test_logout_missing_refresh_token_cookie() {
    let app = get_app_mock().await;

    // Attempt to hit logout without sending any cookies
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/logout")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Expecting 401 Unauthorized because the refresh_token cookie wasn't provided
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body_bytes = to_bytes(response.into_body(), 1024).await.unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert!(
        body_string.contains("You are not logged in, please provide a token")
            || body_string.contains("fail")
    );
}

#[tokio::test]
async fn test_logout_invalid_refresh_token() {
    let app = get_app_mock().await;

    // Pass a fabricated or fake refresh token cookie that doesn't exist in the cache map
    let fake_cookie = "refresh_token=fake_random_token_string_12345;";

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/logout")
                .header("cookie", fake_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Expecting 401 Unauthorized because the token is not present in the refresh_tokens hashmap
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body_bytes = to_bytes(response.into_body(), 1024).await.unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();
    println!("body_string: {:?}", body_string);
    assert!(body_string.contains("Authentication token is invalid or expired") || body_string.contains("fail"));
}
