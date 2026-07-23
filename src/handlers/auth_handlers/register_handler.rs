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