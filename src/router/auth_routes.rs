use axum::{
    Extension, Json, Router,
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
    routing::post,
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use axum_macros::debug_handler;
use validator::Validate;

use crate::{
    AllStates,
    database::users_db::UserExt,
    dtos::user_dtos::{
        FilterUserDto, LoginUserDto, RegisterUserDto, Response, UserLoginResponseDto,
    },
    errors::{ErrorMessage, HttpError},
    utils::{
        password,
        token::{create_main_token, create_refresh_token},
    },
};

pub fn auth_router() -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}

pub async fn register(
    Extension(all_state): Extension<AllStates>,
    Json(body): Json<RegisterUserDto>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let hash_password =
        password::hash(&body.password).map_err(|e| HttpError::server_error(e.to_string()))?;

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

pub async fn login(
    Extension(all_state): Extension<AllStates>,
    Json(body): Json<LoginUserDto>,
) -> Result<impl IntoResponse, HttpError> {
    // Validate the input
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    // Fetch user from the database
    let mut result = all_state
        .app_state
        .db_client
        .get_user(None, None, Some(&body.identifier))
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    if result.is_none() {
        result = all_state
            .app_state
            .db_client
            .get_user(None, Some(&body.identifier), None)
            .await
            .map_err(|e| HttpError::server_error(e.to_string()))?;
    }

    let user = result.ok_or(HttpError::bad_request(
        ErrorMessage::WrongCredentials.to_string(),
    ))?;

    // Compare passwords
    let password_matches = password::compare(&body.password, &user.password_hash)
        .map_err(|_| HttpError::bad_request(ErrorMessage::WrongCredentials.to_string()))?;

    if !password_matches {
        return Err(HttpError::bad_request(
            ErrorMessage::WrongCredentials.to_string(),
        ));
    }
    // Create JWT token
    let token = create_main_token(&user, &all_state)?;

    // Create a refresh token and store it in the hashmap which act as a cache for refresh tokens
    let refresh_token = create_refresh_token(&user, &all_state)?;

    all_state
        .refresh_tokens
        .lock()
        .await
        .insert(refresh_token.clone(), refresh_token.clone());

    let cookie_duration = time::Duration::minutes(all_state.app_state.env.jwt_maxage); // Convert minutes to seconds
    let cookie = Cookie::build(("token", token.clone()))
        .path("/")
        .max_age(cookie_duration)
        .http_only(true)
        .build();

    let refresh_cookie_duration = time::Duration::days(all_state.app_state.env.refresh_jwt_maxage); // Convert months to days
    let refresh_cookie = Cookie::build(("refresh_token", refresh_token.clone()))
        .path("/")
        .max_age(refresh_cookie_duration)
        .http_only(true)
        .build();

    let filter_user = FilterUserDto::filter_user(&user);

    let mut headers = HeaderMap::new();

    headers.append(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    headers.append(
        header::SET_COOKIE,
        refresh_cookie.to_string().parse().unwrap(),
    );

    // Prepare response
    let response = axum::response::Json(UserLoginResponseDto {
        status: "success".to_string(),
        user: filter_user,
        token,
        refresh_token,
    });

    let mut response = response.into_response();
    response.headers_mut().extend(headers);

    Ok(response)
}

#[debug_handler]
pub async fn logout(
    cookie_jar: CookieJar,
    Extension(all_state): Extension<AllStates>,
) -> Result<impl IntoResponse, HttpError> {
    let refresh_token_opt = cookie_jar
        .get("refresh_token")
        .map(|cookie| cookie.value().to_string());

    let refresh_token = match refresh_token_opt {
        Some(refresh_token) => refresh_token,
        None => {
            return Err(HttpError::unauthorized(
                ErrorMessage::TokenNotProvided.to_string(),
            ));
        }
    };

    if !all_state
        .refresh_tokens
        .lock()
        .await
        .contains_key(&refresh_token)
    {
        return Err(HttpError::unauthorized(
            ErrorMessage::InvalidToken.to_string(),
        ));
    }

    all_state.refresh_tokens.lock().await.remove(&refresh_token);

    let cookie_jar = cookie_jar.remove("refresh_token");

    let _ = cookie_jar.remove("token");

    Ok((
        StatusCode::OK,
        Json(Response {
            status: "success",
            message: "Logout successful! User successfully logout".to_string(),
        }),
    ))
}
