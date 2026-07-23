use crate::{
    AllStates,
    db::DatabaseClient,
    dtos::user_dtos::Response,
    errors::{ErrorMessage, HttpError},
};
use axum::{Extension, Json, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

pub async fn logout<T>(
    cookie_jar: CookieJar,
    Extension(all_state): Extension<AllStates<T>>,
) -> Result<impl IntoResponse, HttpError>
where
    T: DatabaseClient + Clone + 'static,
{
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

    let cookie_jar = cookie_jar.remove("refresh_token").remove("token");

    Ok((
        StatusCode::OK,
        cookie_jar,
        Json(Response {
            status: "success",
            message: "Logout successful! User successfully logout".to_string(),
        }),
    ))
}
