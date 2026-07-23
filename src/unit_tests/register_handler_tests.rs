#[cfg(test)]
mod register_handler_tests {
    use crate::{
        AllStates, AppState,
        config::{Config, ConfigMockExt},
        db::DBClientMock,
        dtos::user_dtos::RegisterUserDto,
        handlers::auth_handlers::register_handler::register,
    };
    use axum::{Extension, Json, http::StatusCode, response::IntoResponse};
    use std::{collections::HashMap, sync::Arc};
    use tokio::sync::Mutex;
    // Import your mock database client or whatever implements DatabaseClient
    // e.g., use crate::common::db_mock::DBClientMock; (adjust path to your project structure)

    #[tokio::test]
    async fn register_test() {
        // 1. Set up dummy configuration and mock database client state
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

        // 2. Prepare a valid RegisterUserDto payload
        let payload = RegisterUserDto {
            username: "unittestuser".to_string(),
            email: "unittest@example.com".to_string(),
            password: "SecurePassword123!".to_string(),
            password_confirm: "SecurePassword123!".to_string(),
        };

        // 3. Call the register handler directly
        let response_result = register(Extension(all_state), Json(payload)).await;

        // 4. Assert success response
        assert!(
            response_result.is_ok(),
            "Registration failed with error: {:?}",
            response_result.err()
        );

        let response = response_result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_register_username_already_exists() {
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

        // First registration
        let payload1 = RegisterUserDto {
            username: "samename".to_string(),
            email: "email1@example.com".to_string(),
            password: "SecurePassword123!".to_string(),
            password_confirm: "SecurePassword123!".to_string(),
        };
        let _ = register(Extension(all_state.clone()), Json(payload1)).await;

        // Second registration with the same username but different email
        let payload2 = RegisterUserDto {
            username: "samename".to_string(),
            email: "email2@example.com".to_string(),
            password: "SecurePassword123!".to_string(),
            password_confirm: "SecurePassword123!".to_string(),
        };

        let response_result = register(Extension(all_state), Json(payload2)).await;

        assert!(
            response_result.is_err(),
            "Expected unique constraint error for username"
        );
    }

    #[tokio::test]
    async fn test_register_email_already_exists() {
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

        // First registration
        let payload1 = RegisterUserDto {
            username: "userone".to_string(),
            email: "sameemail@example.com".to_string(),
            password: "SecurePassword123!".to_string(),
            password_confirm: "SecurePassword123!".to_string(),
        };
        let _ = register(Extension(all_state.clone()), Json(payload1)).await;

        // Second registration with a different username but the same email
        let payload2 = RegisterUserDto {
            username: "usertwo".to_string(),
            email: "sameemail@example.com".to_string(),
            password: "SecurePassword123!".to_string(),
            password_confirm: "SecurePassword123!".to_string(),
        };

        let response_result = register(Extension(all_state), Json(payload2)).await;

        assert!(
            response_result.is_err(),
            "Expected unique constraint error for email"
        );
    }

    #[tokio::test]
    async fn test_register_invalid_email_format() {
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

        let payload = RegisterUserDto {
            username: "validuser".to_string(),
            email: "not-an-email-address".to_string(), // Invalid email format
            password: "SecurePassword123!".to_string(),
            password_confirm: "SecurePassword123!".to_string(),
        };

        let response_result = register(Extension(all_state), Json(payload)).await;

        assert!(
            response_result.is_err(),
            "Expected validation failure for bad email format"
        );
    }

    #[tokio::test]
    async fn test_register_weak_password() {
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

        let payload = RegisterUserDto {
            username: "validuser".to_string(),
            email: "user@example.com".to_string(),
            password: "123".to_string(), // Weak/short password
            password_confirm: "123".to_string(),
        };

        let response_result = register(Extension(all_state), Json(payload)).await;

        assert!(
            response_result.is_err(),
            "Expected validation failure for weak password"
        );
    }

    #[tokio::test]
    async fn test_register_passwords_do_not_match() {
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

        let payload = RegisterUserDto {
            username: "validuser".to_string(),
            email: "user@example.com".to_string(),
            password: "SecurePassword123!".to_string(),
            password_confirm: "DifferentPassword123!".to_string(), // Mismatched confirmation
        };

        let response_result = register(Extension(all_state), Json(payload)).await;

        assert!(
            response_result.is_err(),
            "Expected validation failure when passwordConfirm doesn't match"
        );
    }
}
