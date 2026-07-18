use anyhow::{Result, anyhow};
use axum_auth_v2::config::Config;
use axum_auth_v2::db::DBClient;
use axum_auth_v2::router::create_routes;

use axum::{body::Body, extract::Request, http::StatusCode};
use axum_auth_v2::{AllStates, AppState, get_database_pool};
use tower::ServiceExt;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
// You might need to make these fields/structs public or create a test helper in your main crate

async fn create_mock_state() -> Result<AllStates> {
    // You'll need access to Config::default() or build a dummy one
    let dummy_config = Config::init()?;

      let pool = get_database_pool(&dummy_config)
        .await
        .map_err(|e| anyhow!("Fail to get database pool: {e}"))?;

    // Create an uninitialized/mock DBClient if possible
    let dummy_db = DBClient::new(pool);

    Ok(AllStates {
        app_state: Arc::new(AppState {
            env: dummy_config,
            db_client: dummy_db,
        }),
        refresh_tokens: Arc::new(Mutex::new(HashMap::new())),
    })
}

#[tokio::test]
async fn test_home_route() {
    let mock_state = create_mock_state().await
        .map_err(|e| {
            let text = anyhow!("Error: {e}");
            panic!("{:?}", text);
        })
        .unwrap();
    let app = create_routes(mock_state);

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_authorized_route_fails_without_auth() {
    let mock_state = create_mock_state().await
        .map_err(|e| {
            let text = anyhow!("Error: {e}");
            panic!("{:?}", text);
        })
        .unwrap();
    let app = create_routes(mock_state);

    // Assuming authorized routes are under /api/logout or similar
    // Since you nested authorized_api directly into api_route,
    // they are likely at the root of /api/
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/logout")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should be 401 Unauthorized because of the 'auth' middleware
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_route_exists() {
    let mock_state = create_mock_state().await
        .map_err(|e| {
            let text = anyhow!("Error: {e}");
            panic!("{:?}", text);
        })
        .unwrap();
    let app = create_routes(mock_state);

    // Test that the /auth path is reachable (assuming it's not protected)
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/auth")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Depending on your implementation, this might be 200 or 404/405
    assert_ne!(response.status(), StatusCode::NOT_FOUND);
}
