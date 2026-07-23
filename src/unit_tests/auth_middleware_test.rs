#[cfg(test)]
mod auth_middleware_test {
    use crate::{
        AllStates, AppState,
        config::{Config, ConfigMockExt},
        database::users_db::UserExt,
        db::{DBClientMock, DatabaseClient},
        middlewares::auth_middleware::{JWTAuthMiddleware, auth},
        utils::token,
    };
    use axum::{
        Extension, Router,
        body::Body,
        http::{Request, StatusCode, header},
        middleware::{self},
        response::IntoResponse,
        routing::get,
    };
    use std::{collections::HashMap, sync::Arc};
    use tokio::sync::Mutex;
    use tower::ServiceExt;

    // Helper dummy handler to simulate an incoming protected route request passing through the middleware
    async fn dummy_protected_handler(
        Extension(auth_data): Extension<JWTAuthMiddleware>,
    ) -> impl IntoResponse {
        format!("Welcome, {}", auth_data.user.username)
    }

    fn setup_test_app<T>(all_state: AllStates<T>) -> Router
    where
        T: DatabaseClient + Clone + Send + Sync + 'static,
    {
        Router::new()
            .route("/protected", get(dummy_protected_handler))
            .layer(middleware::from_fn(auth::<T>))
            .layer(Extension(all_state))
    }

    #[tokio::test]
    async fn test_auth_success_with_main_token_cookie() {
        let dummy_config = Config::mock().unwrap();
        let dummy_db = Arc::new(DBClientMock::mock());

        let app_state = Arc::new(AppState {
            env: dummy_config,
            db_client: dummy_db.clone(),
        });

        let all_state = AllStates {
            app_state,
            refresh_tokens: Arc::new(Mutex::new(HashMap::new())),
        };

        // Seed user and generate main token
        let hashed_password = crate::utils::password::hash("SecurePassword123!").unwrap();
        let user = dummy_db
            .save_user("cookietest", "cookie@example.com", &hashed_password)
            .await
            .unwrap();
        let main_token = token::create_main_token(&user, &all_state).unwrap();

        let app = setup_test_app(all_state);

        // Send request with main token in cookie jar
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("cookie", format!("token={}", main_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_success_with_bearer_authorization_header() {
        let dummy_config = Config::mock().unwrap();
        let dummy_db = Arc::new(DBClientMock::mock());

        let app_state = Arc::new(AppState {
            env: dummy_config,
            db_client: dummy_db.clone(),
        });

        let all_state = AllStates {
            app_state,
            refresh_tokens: Arc::new(Mutex::new(HashMap::new())),
        };

        let hashed_password = crate::utils::password::hash("SecurePassword123!").unwrap();
        let user = dummy_db
            .save_user("bearertest", "bearer@example.com", &hashed_password)
            .await
            .unwrap();
        let main_token = token::create_main_token(&user, &all_state).unwrap();

        let app = setup_test_app(all_state);

        // Send request with Bearer token in header instead of cookie
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("authorization", format!("Bearer {}", main_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_success_fallback_to_refresh_token() {
        let dummy_config = Config::mock().unwrap();
        let dummy_db = Arc::new(DBClientMock::mock());

        let app_state = Arc::new(AppState {
            env: dummy_config,
            db_client: dummy_db.clone(),
        });

        let refresh_tokens_map = Arc::new(Mutex::new(HashMap::new()));
        let all_state = AllStates {
            app_state,
            refresh_tokens: refresh_tokens_map.clone(),
        };

        let hashed_password = crate::utils::password::hash("SecurePassword123!").unwrap();
        let user = dummy_db
            .save_user("reftokenuser", "reftoken@example.com", &hashed_password)
            .await
            .unwrap();

        // Generate and cache refresh token, while main token is absent/invalid
        let refresh_token = token::create_refresh_token(&user, &all_state).unwrap();
        refresh_tokens_map
            .lock()
            .await
            .insert(refresh_token.clone(), refresh_token.clone());

        let app = setup_test_app(all_state);

        // Send request with ONLY the refresh_token cookie
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("cookie", format!("refresh_token={}", refresh_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should succeed by falling back to refresh token, validating it, and issuing a new token cookie
        assert_eq!(response.status(), StatusCode::OK);

        let set_cookie = response.headers().get(header::SET_COOKIE);
        assert!(
            set_cookie.is_some(),
            "Expected a new token cookie to be appended on refresh fallback"
        );
    }

    #[tokio::test]
    async fn test_auth_unauthorized_missing_all_tokens() {
        let dummy_config = Config::mock().unwrap();
        let dummy_db = Arc::new(DBClientMock::mock());

        let app_state = Arc::new(AppState {
            env: dummy_config,
            db_client: dummy_db,
        });

        let all_state = AllStates {
            app_state,
            refresh_tokens: Arc::new(Mutex::new(HashMap::new())),
        };

        let app = setup_test_app(all_state);

        // Send request with no tokens whatsoever
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_unauthorized_user_no_longer_exists() {
        let dummy_config = Config::mock().unwrap();
        let dummy_db = Arc::new(DBClientMock::mock());

        let app_state = Arc::new(AppState {
            env: dummy_config,
            db_client: dummy_db.clone(),
        });

        let all_state = AllStates {
            app_state,
            refresh_tokens: Arc::new(Mutex::new(HashMap::new())),
        };

        let hashed_password = crate::utils::password::hash("SecurePassword123!").unwrap();
        let temp_user = dummy_db
            .save_user("ghost", "ghost@example.com", &hashed_password)
            .await
            .unwrap();

        // Create a token pointing to a user ID that doesn't match a stored record
        let mut orphan_user = temp_user;
        orphan_user.id = uuid::Uuid::new_v4();
        let invalid_token = token::create_main_token(&orphan_user, &all_state).unwrap();

        let app = setup_test_app(all_state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("cookie", format!("token={}", invalid_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
