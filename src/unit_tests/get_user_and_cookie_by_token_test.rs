#[cfg(test)]
mod get_user_and_cookie_by_token {
    use crate::{
        AllStates, AppState,
        config::{Config, ConfigMockExt},
        database::users_db::UserExt,
        db::DBClientMock,
        errors::ErrorMessage,
        middlewares::auth_middleware::user_and_cookie_by_token::get_user_and_cookie_by_token,
    };
    use std::{collections::HashMap, sync::Arc};
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_get_user_by_token_success() {
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
        let saved_user = dummy_db
            .save_user("tokenuser", "tokenuser@example.com", &hashed_password)
            .await
            .unwrap();

        let valid_token = crate::utils::token::create_main_token(&saved_user, &all_state).unwrap();

        let result = get_user_and_cookie_by_token(Some(valid_token), &all_state).await;

        assert!(result.is_ok(), "Expected success, got: {:?}", result.err());
        let user_and_cookie = result.unwrap();
        assert_eq!(user_and_cookie.user.id, saved_user.id);
        assert!(user_and_cookie.cookie.is_none());
    }

    #[tokio::test]
    async fn test_get_user_by_token_missing() {
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

        let result = get_user_and_cookie_by_token(None, &all_state).await;

        assert_eq!(result, Err(ErrorMessage::TokenNotProvided));
    }

    #[tokio::test]
    async fn test_get_user_by_token_invalid_signature_or_malformed() {
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

        let result =
            get_user_and_cookie_by_token(Some("not.a.real.jwt.token".to_string()), &all_state)
                .await;

        assert_eq!(result, Err(ErrorMessage::InvalidToken));
    }

    #[tokio::test]
    async fn test_get_user_by_token_user_no_longer_exists() {
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

        // Save a temporary user to get a validly structured token,
        // but then we can test with a token generated for a completely random user ID
        // that was never inserted into the database.
        let temp_user = dummy_db
            .save_user("tempuser", "temp@example.com", &hashed_password)
            .await
            .unwrap();

        // Mutate the ID of the user object to a brand-new random UUID that doesn't exist in the mock DB
        let mut orphan_user = temp_user;
        orphan_user.id = uuid::Uuid::new_v4();

        let orphan_token =
            crate::utils::token::create_main_token(&orphan_user, &all_state).unwrap();

        let result = get_user_and_cookie_by_token(Some(orphan_token), &all_state).await;

        assert_eq!(result, Err(ErrorMessage::UserNoLongerExist));
    }
}
