#[cfg(test)]
mod get_user_and_cookie_by_refresh_token_tests {
    use crate::{
        AllStates, AppState,
        config::{Config, ConfigMockExt},
        database::users_db::UserExt,
        db::DBClientMock,
        middlewares::auth_middleware::user_and_cookie_by_refresh_token::get_user_and_cookie_by_refresh_token,
        utils::token,
    };
    use std::{collections::HashMap, sync::Arc};
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_get_user_by_refresh_token_success() {
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

        // 1. Seed a user in the mock DB
        let hashed_password = crate::utils::password::hash("SecurePassword123!").unwrap();
        let saved_user = dummy_db
            .save_user("refuser", "refuser@example.com", &hashed_password)
            .await
            .unwrap();

        // 2. Generate a valid refresh token using token utility
        let refresh_token = token::create_refresh_token(&saved_user, &all_state).unwrap();

        // 3. Insert the refresh token into the server-side cache (hashmap)
        refresh_tokens_map
            .lock()
            .await
            .insert(refresh_token.clone(), refresh_token.clone());

        // 4. Call the function
        let result = get_user_and_cookie_by_refresh_token(refresh_token, all_state).await;

        assert!(result.is_ok(), "Expected success, got: {:?}", result.err());

        let user_and_cookie = result.unwrap();
        
        assert_eq!(user_and_cookie.user.id, saved_user.id);
        
        assert!(
            user_and_cookie.cookie.is_some(),
            "Expected a new token cookie to be returned"
        );

        let cookie = user_and_cookie.cookie.unwrap();
        assert_eq!(cookie.name(), "token");
    }

    #[tokio::test]
    async fn test_get_user_by_refresh_token_not_in_cache() {
        let dummy_config = Config::mock().unwrap();
        let dummy_db = Arc::new(DBClientMock::mock());

        let app_state = Arc::new(AppState {
            env: dummy_config,
            db_client: dummy_db.clone(),
        });

        let all_state = AllStates {
            app_state,
            refresh_tokens: Arc::new(Mutex::new(HashMap::new())), // Cache is empty
        };

        // Pass a refresh token that was never cached/stored on the server
        let result = get_user_and_cookie_by_refresh_token(
            "untracked_refresh_token_string".to_string(),
            all_state,
        )
        .await;

        assert!(
            result.is_err(),
            "Expected error because token is not in cache"
        );
    }

    #[tokio::test]
    async fn test_get_user_by_refresh_token_invalid_jwt_string() {
        let dummy_config = Config::mock().unwrap();
        let dummy_db = Arc::new(DBClientMock::mock());

        let app_state = Arc::new(AppState {
            env: dummy_config,
            db_client: dummy_db.clone(),
        });

        let refresh_tokens_map = Arc::new(Mutex::new(HashMap::new()));
        let fake_token = "malformed.jwt.string".to_string();

        // Insert the malformed string into cache so it passes the first check
        refresh_tokens_map
            .lock()
            .await
            .insert(fake_token.clone(), fake_token.clone());

        let all_state = AllStates {
            app_state,
            refresh_tokens: refresh_tokens_map,
        };

        let result = get_user_and_cookie_by_refresh_token(fake_token, all_state).await;

        assert!(
            result.is_err(),
            "Expected error due to invalid token decoding"
        );
    }

    #[tokio::test]
    async fn test_get_user_by_refresh_token_user_no_longer_exists() {
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
        let temp_user = dummy_db
            .save_user("ghostref", "ghostref@example.com", &hashed_password)
            .await
            .unwrap();

        // Mutate ID so it looks validly signed, but points to a user ID not in the database
        let mut orphan_user = temp_user;
        orphan_user.id = uuid::Uuid::new_v4();
        let orphan_refresh_token = token::create_refresh_token(&orphan_user, &all_state).unwrap();

        // Put orphan token in cache
        refresh_tokens_map
            .lock()
            .await
            .insert(orphan_refresh_token.clone(), orphan_refresh_token.clone());

        let result = get_user_and_cookie_by_refresh_token(orphan_refresh_token, all_state).await;

        assert!(
            result.is_err(),
            "Expected error because user does not exist in database"
        );
    }
}
