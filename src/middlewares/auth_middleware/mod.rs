pub mod user_and_cookie_by_refresh_token;
pub mod user_and_cookie_by_token;

use axum::{Extension, extract::Request, http::header, middleware::Next, response::IntoResponse};

use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::{Deserialize, Serialize};

use crate::{
    AllStates,
    db::DatabaseClient,
    errors::{ErrorMessage, HttpError},
    middlewares::auth_middleware::{
        user_and_cookie_by_refresh_token::get_user_and_cookie_by_refresh_token,
        user_and_cookie_by_token::get_user_and_cookie_by_token,
    },
    models::user_model::User,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JWTAuthMiddleware {
    pub user: User,
}

#[derive(Debug, PartialEq, Clone)]
pub struct UserAndCookie {
    pub user: User,
    pub cookie: Option<Cookie<'static>>,
}

pub async fn auth<T>(
    cookie_jar: CookieJar,
    Extension(all_state): Extension<AllStates<T>>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, HttpError>
where
    T: DatabaseClient + Clone + 'static,
{
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

    let user_and_cookie = get_user_and_cookie_by_token(cookies, &all_state).await;

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

        let res = get_user_and_cookie_by_refresh_token(token, all_state).await?;
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
