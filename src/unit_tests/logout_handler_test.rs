#[cfg(test)]
mod logout_handler_test {
    use crate::{
        AllStates, AppState,
        config::{Config, ConfigMockExt},
        db::DBClientMock,
        handlers::authorized_handlers::logout_handler::logout,
    };
    use axum::{Extension, http::StatusCode, response::IntoResponse};
    use axum_extra::extract::{CookieJar, cookie::Cookie};
    use std::{collections::HashMap, sync::Arc};
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_logout_success() {
        let dummy_config = Config::mock().unwrap();
        let dummy_db = Arc::new(DBClientMock::mock());

        let app_state = Arc::new(AppState {
            env: dummy_config,
            db_client: dummy_db,
        });

        let refresh_tokens_map = Arc::new(Mutex::new(HashMap::new()));

        // Pre-populate the cache with a valid refresh token
        let test_token = "valid_refresh_token_12345".to_string();
        refresh_tokens_map
            .lock()
            .await
            .insert(test_token.clone(), test_token.clone());

        let all_state = AllStates {
            app_state,
            refresh_tokens: refresh_tokens_map.clone(),
        };

        // Construct a CookieJar containing the valid refresh token
        let mut cookie_jar = CookieJar::new();
        cookie_jar = cookie_jar.add(Cookie::new("refresh_token", test_token.clone()));

        // Call the logout handler directly
        let response_result = logout(cookie_jar, Extension(all_state)).await;

        assert!(
            response_result.is_ok(),
            "Logout failed with error: {:?}",
            response_result.err()
        );

        let response = response_result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::OK);

        // Verify that the token was successfully removed from the server cache
        let is_still_cached = refresh_tokens_map.lock().await.contains_key(&test_token);
        assert!(
            !is_still_cached,
            "Refresh token should have been removed from the hashmap cache"
        );
    }

    #[tokio::test]
    async fn test_logout_missing_refresh_token_cookie() {
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

        // Empty CookieJar (no cookies provided at all)
        let cookie_jar = CookieJar::new();

        let response_result = logout(cookie_jar, Extension(all_state)).await;

        // Expecting an unauthorized error because token wasn't provided
        assert!(
            response_result.is_err(),
            "Expected error when refresh_token cookie is missing"
        );
    }

    #[tokio::test]
    async fn test_logout_invalid_refresh_token() {
        let dummy_config = Config::mock().unwrap();
        let dummy_db = Arc::new(DBClientMock::mock());

        let app_state = Arc::new(AppState {
            env: dummy_config,
            db_client: dummy_db,
        });

        let all_state = AllStates {
            app_state,
            refresh_tokens: Arc::new(Mutex::new(HashMap::new())), // Cache is empty
        };

        // Provide a refresh token that does not exist in the hashmap cache
        let mut cookie_jar = CookieJar::new();
        cookie_jar = cookie_jar.add(Cookie::new("refresh_token", "fake_untracked_token"));

        let response_result = logout(cookie_jar, Extension(all_state)).await;

        // Expecting an unauthorized error because token is invalid/not found in cache
        assert!(
            response_result.is_err(),
            "Expected error for untracked/invalid refresh token"
        );
    }
}
