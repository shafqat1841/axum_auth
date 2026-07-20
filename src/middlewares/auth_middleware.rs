use axum::{Extension, extract::Request, http::header, middleware::Next, response::IntoResponse};

use axum_extra::extract::cookie::{Cookie, CookieJar};
use axum_macros::debug_middleware;
use serde::{Deserialize, Serialize};

use crate::{
    AllStatesDBClient,
    database::users_db::UserExt,
    errors::{ErrorMessage, HttpError},
    models::user_model::User,
    utils::token::{self, create_main_token},
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
    all_state: &AllStatesDBClient,
) -> Result<UserAndCookie, ErrorMessage> {
    let app_state = &all_state.app_state;
    let token_details = match token::decode_token(token, app_state.env.jwt_secret.as_bytes()) {
        Ok(token_details) => token_details,
        Err(_) => {
            return Err(ErrorMessage::InvalidToken);
        }
    };

    let user_id = uuid::Uuid::parse_str(&token_details.to_string());

    let user_id = match user_id {
        Ok(user_id) => user_id,
        Err(_) => return Err(ErrorMessage::InvalidToken),
    };

    // Fetch user from database
    let user = app_state
        .db_client
        .get_user(Some(user_id), None, None)
        .await;

    let user = match user {
        Ok(user) => user,
        Err(_) => {
            return Err(ErrorMessage::UserNoLongerExist);
        }
    };

    let user = match user {
        None => {
            return Err(ErrorMessage::UserNoLongerExist);
        }
        Some(user) => user,
    };

    Ok(UserAndCookie { user, cookie: None })
}

async fn work_on_refresh_token(
    refresh_token: String,
    all_state: AllStatesDBClient,
) -> Result<UserAndCookie, HttpError> {
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

// Middleware function for role-based authorization
#[debug_middleware]
pub async fn auth(
    cookie_jar: CookieJar,
    Extension(all_state): Extension<AllStatesDBClient>,
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

    let token = match cookies {
        Some(token) => token,
        None => {
            return Err(HttpError::unauthorized(
                ErrorMessage::TokenNotProvided.to_string(),
            ));
        }
    };

    let user_and_cookie = work_on_token(token, &all_state).await;

    let mut user_and_cookie = match user_and_cookie {
        Err(err) => match err {
            ErrorMessage::UserNoLongerExist => {
                return Err(HttpError::unauthorized(
                    ErrorMessage::UserNoLongerExist.to_string(),
                ));
            }
            _ => None,
        },

        Ok(user_and_cookie) => Some(user_and_cookie),
    };

    if let None = user_and_cookie {
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

        let res = work_on_refresh_token(token, all_state).await?;
        user_and_cookie = Some(res)
    }

    match user_and_cookie {
        None => {
            return Err(HttpError::unauthorized(
                ErrorMessage::InvalidToken.to_string(),
            ));
        }
        Some(user_and_cookie) => {
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
    }
}
