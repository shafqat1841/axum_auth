use crate::{
    AllStates,
    db::DatabaseClient,
    dtos::user_dtos::{FilterUserDto, LoginUserDto, UserLoginResponseDto},
    errors::{ErrorMessage, HttpError},
    utils::{
        password,
        token::{create_main_token, create_refresh_token},
    },
};
use axum::{
    Extension, Json,
    http::{HeaderMap, header},
    response::IntoResponse,
};
use axum_extra::extract::cookie::Cookie;
use validator::Validate;

pub async fn login<T>(
    Extension(all_state): Extension<AllStates<T>>,
    Json(body): Json<LoginUserDto>,
) -> Result<impl IntoResponse, HttpError>
where
    T: DatabaseClient + Clone + 'static,
{
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
