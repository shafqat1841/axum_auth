use axum_extra::extract::cookie::Cookie;

use crate::{
    AllStates,
    db::DatabaseClient,
    errors::{ErrorMessage, HttpError},
    middlewares::auth_middleware::UserAndCookie,
    utils::token::{self, create_main_token},
};

pub async fn get_user_and_cookie_by_refresh_token<T>(
    refresh_token: String,
    all_state: AllStates<T>,
) -> Result<UserAndCookie, HttpError>
where
    T: DatabaseClient + Clone + 'static,
{
    let app_state = &all_state.app_state;

    if !&all_state
        .refresh_tokens
        .lock()
        .await
        .contains_key(&refresh_token)
    {
        return Err(HttpError::unauthorized(
            ErrorMessage::InvalidToken.to_string(),
        ));
    }

    let token_details =
        match token::decode_token(refresh_token, app_state.env.refresh_jwt_secret.as_bytes()) {
            Ok(token_details) => token_details,
            Err(_) => {
                return Err(HttpError::unauthorized(
                    ErrorMessage::InvalidToken.to_string(),
                ));
            }
        };

    let user_id = uuid::Uuid::parse_str(&token_details.to_string())
        .map_err(|_| HttpError::unauthorized(ErrorMessage::InvalidToken.to_string()))?;

    // Fetch user from database
    let user = app_state
        .db_client
        .get_user(Some(user_id), None, None)
        .await
        .map_err(|_| HttpError::unauthorized(ErrorMessage::UserNoLongerExist.to_string()))?;

    let user =
        user.ok_or_else(|| HttpError::unauthorized(ErrorMessage::UserNoLongerExist.to_string()))?;

    // Create JWT token
    let token = create_main_token(&user, &all_state)?;

    let cookie_duration = time::Duration::minutes(app_state.env.jwt_maxage); // Convert minutes to seconds
    let cookie: Cookie<'_> = Cookie::build(("token", token.clone()))
        .path("/")
        .max_age(cookie_duration)
        .http_only(true)
        .build();

    Ok(UserAndCookie {
        user,
        cookie: Some(cookie),
    })
}
