use axum::{Extension, extract::Request, http::header, middleware::Next, response::IntoResponse};

use axum_extra::extract::cookie::{Cookie, CookieJar};
use axum_macros::debug_middleware;
use serde::{Deserialize, Serialize};

use crate::{
    AllStates,
    database::users_db::UserExt,
    errors::{ErrorMessage, HttpError},
    models::user_model::User,
    utils::token,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JWTAuthMiddleware {
    pub user: User,
}

struct UserAndCookie {
    pub user: User,
    pub cookie: Option<Cookie<'static>>,
}

async fn work_on_token(
    token: String,
    all_state: AllStates,
    // mut req: Request,
    // next: Next,
) -> Result<UserAndCookie, HttpError> {
    let app_state = all_state.app_state;
    let token_details = match token::decode_token(token, app_state.env.jwt_secret.as_bytes()) {
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

    Ok(UserAndCookie { user, cookie: None })
}

async fn work_on_refresh_token(
    refresh_token: String,
    all_state: AllStates,
    // mut req: Request,
    // next: Next,
) -> Result<UserAndCookie, HttpError> {
    let app_state = all_state.app_state;

    let refresh_tokens_lock = all_state.refresh_tokens.lock();

    if !refresh_tokens_lock.await.contains_key(&refresh_token) {
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
    let token = token::create_token(
        &user.id.to_string(),
        app_state.env.jwt_secret.as_bytes(),
        app_state.env.jwt_maxage,
    )
    .map_err(|e| HttpError::server_error(e.to_string()))?;

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

// Middleware function for role-based authorization
#[debug_middleware]
pub async fn auth(
    cookie_jar: CookieJar,
    Extension(all_state): Extension<AllStates>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, HttpError> {
    // Extract access token from cookie or Authorization header
    let cookies = cookie_jar
        .get("token")
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    if auth_value.starts_with("Bearer ") {
                        Some(auth_value[7..].to_owned())
                    } else {
                        None
                    }
                })
        });

    let user_and_cookie = match cookies {
        Some(token) => {
            let user_and_cookie = work_on_token(token, all_state).await?;
            user_and_cookie
        }
        None => {
            let refresh_token_opt = cookie_jar
                .get("refresh_token")
                .map(|cookie| cookie.value().to_string());

            let token = match refresh_token_opt {
                Some(refresh_token) => refresh_token,
                None => {
                    return Err(HttpError::unauthorized(
                        ErrorMessage::TokenNotProvided.to_string(),
                    ));
                }
            };

            let user_and_cookie = work_on_refresh_token(token, all_state).await?;

            user_and_cookie
        }
    };

    req.extensions_mut().insert(JWTAuthMiddleware {
        user: user_and_cookie.user.clone(),
    });
    let mut response = next.run(req).await;
    if let Some(cookie) = user_and_cookie.cookie {
        response
            .headers_mut()
            .append(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    }

    Ok(response)
}
