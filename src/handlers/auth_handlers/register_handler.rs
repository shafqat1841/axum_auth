use crate::{
    AllStates,
    db::DatabaseClient,
    dtos::user_dtos::{RegisterUserDto, Response},
    errors::{ErrorMessage, HttpError},
    utils::password,
};
use axum::{Extension, Json, http::StatusCode, response::IntoResponse};
use validator::Validate;

pub async fn register<T>(
    Extension(all_state): Extension<AllStates<T>>,
    Json(body): Json<RegisterUserDto>,
) -> Result<impl IntoResponse, HttpError>
where
    T: DatabaseClient + Clone + 'static,
{
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let hash_password =
        password::hash(&body.password).map_err(|e| HttpError::server_error(e.to_string()))?;

    let found_result = all_state
        .app_state
        .db_client
        .get_user(None, Some(&body.username), None)
        .await
        .map_err(|e| {
            let http_err = HttpError::unique_constraint_violation(e.to_string());
            http_err
        })?;

    if let Some(_) = found_result {
        return Err(HttpError::unique_constraint_violation(
            ErrorMessage::UsernameExist.to_string(),
        ));
    }

    let found_result = all_state
        .app_state
        .db_client
        .get_user(None, None, Some(&body.email))
        .await
        .map_err(|e| {
            let http_err = HttpError::unique_constraint_violation(e.to_string());
            http_err
        })?;

    if let Some(_) = found_result {
        return Err(HttpError::unique_constraint_violation(
            ErrorMessage::EmailExist.to_string(),
        ));
    }

    let result = all_state
        .app_state
        .db_client
        .save_user(&body.username, &body.email, &hash_password)
        .await;

    match result {
        Ok(_user) => Ok((
            StatusCode::CREATED,
            Json(Response {
                status: "success",
                message: "Registration successful! You can now log in to your account.".to_string(),
            }),
        )),
        Err(sqlx::Error::Database(db_err)) => {
            if db_err.is_unique_violation() {
                let constraint = db_err.constraint().unwrap_or_default();

                if constraint.contains("email") {
                    Err(HttpError::unique_constraint_violation(
                        ErrorMessage::EmailExist.to_string(),
                    ))
                } else if constraint.contains("username") {
                    Err(HttpError::unique_constraint_violation(
                        ErrorMessage::UsernameExist.to_string(),
                    ))
                } else {
                    Err(HttpError::server_error(
                        "Unique constraint violation".to_string(),
                    ))
                }
            } else {
                Err(HttpError::server_error(db_err.to_string()))
            }
        }
        Err(e) => Err(HttpError::server_error(e.to_string())),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashMap, sync::Arc};
    use tokio::sync::Mutex;
    use crate::{AppState, config::{Config,ConfigMockExt}, db::DBClientMock};
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
        let response_result = register(
            Extension(all_state),
            Json(payload),
        ).await;

        // 4. Assert success response
        assert!(response_result.is_ok(), "Registration failed with error: {:?}", response_result.err());
        
        let response = response_result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::CREATED);
    }
}
