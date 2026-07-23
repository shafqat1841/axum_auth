#[cfg(test)]
mod login_handler_test {

    use axum::{
        Extension, Json,
        http::{StatusCode, header},
        response::IntoResponse,
    };
    use std::{collections::HashMap, sync::Arc};
    use tokio::sync::Mutex;

    use crate::{
        AllStates, AppState,
        config::{Config, ConfigMockExt},
        database::users_db::UserExt,
        db::DBClientMock,
        dtos::user_dtos::LoginUserDto,
        handlers::auth_handlers::login_handler::login,
        utils::password,
    };

    #[tokio::test]
    async fn test_login_success_with_email() {
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

        let hashed_password = password::hash("SecurePassword123!").unwrap();
        let _ = all_state
            .app_state
            .db_client
            .save_user("logintestuser", "logintest@example.com", &hashed_password)
            .await;

        let payload = LoginUserDto {
            identifier: "logintest@example.com".to_string(),
            password: "SecurePassword123!".to_string(),
        };

        let response_result = login(Extension(all_state), Json(payload)).await;

        assert!(
            response_result.is_ok(),
            "Login failed with error: {:?}",
            response_result.err()
        );

        let response = response_result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::OK);

        // Verify cookies are present in the response headers
        let set_cookie_headers: Vec<_> = response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|h| h.to_str().unwrap())
            .collect();

        assert!(set_cookie_headers.iter().any(|c| c.contains("token=")));
        assert!(
            set_cookie_headers
                .iter()
                .any(|c| c.contains("refresh_token="))
        );
    }

    #[tokio::test]
    async fn test_login_success_with_username() {
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

        let hashed_password = password::hash("SecurePassword123!").unwrap();
        let _ = all_state
            .app_state
            .db_client
            .save_user("logintestuser", "logintest@example.com", &hashed_password)
            .await;

        // Using username as the identifier instead of email
        let payload = LoginUserDto {
            identifier: "logintestuser".to_string(),
            password: "SecurePassword123!".to_string(),
        };

        let response_result = login(Extension(all_state), Json(payload)).await;

        assert!(response_result.is_ok());
        let response = response_result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_login_user_not_found() {
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

        let payload = LoginUserDto {
            identifier: "nonexistent@example.com".to_string(),
            password: "SecurePassword123!".to_string(),
        };

        let response_result = login(Extension(all_state), Json(payload)).await;

        // Expect error (WrongCredentials mapped to Bad Request)
        assert!(response_result.is_err());
    }

    #[tokio::test]
    async fn test_login_wrong_password() {
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

        let hashed_password = password::hash("SecurePassword123!").unwrap();
        let _ = all_state
            .app_state
            .db_client
            .save_user("logintestuser", "logintest@example.com", &hashed_password)
            .await;

        let payload = LoginUserDto {
            identifier: "logintest@example.com".to_string(),
            password: "WrongPassword999!".to_string(), // Incorrect password
        };

        let response_result = login(Extension(all_state), Json(payload)).await;

        assert!(
            response_result.is_err(),
            "Expected error due to incorrect password"
        );
    }

    #[tokio::test]
    async fn test_login_invalid_identifier_format() {
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

        // Supplying empty or structurally invalid fields according to validation rules
        let payload = LoginUserDto {
            identifier: "".to_string(), // Empty identifier
            password: "SecurePassword123!".to_string(),
        };

        let response_result = login(Extension(all_state), Json(payload)).await;

        assert!(
            response_result.is_err(),
            "Expected validation failure for empty identifier"
        );
    }

    #[tokio::test]
    async fn test_login_empty_password() {
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

        let payload = LoginUserDto {
            identifier: "logintest@example.com".to_string(),
            password: "".to_string(), // Empty password
        };

        let response_result = login(Extension(all_state), Json(payload)).await;

        assert!(
            response_result.is_err(),
            "Expected validation failure for empty password"
        );
    }
}
