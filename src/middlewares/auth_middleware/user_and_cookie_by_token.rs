use crate::{
    AllStates, db::DatabaseClient, errors::ErrorMessage,
    middlewares::auth_middleware::UserAndCookie, utils::token,
};

pub async fn get_user_and_cookie_by_token<T>(
    main_token: Option<String>,
    all_state: &AllStates<T>,
) -> Result<UserAndCookie, ErrorMessage>
where
    T: DatabaseClient + Clone + 'static,
{
    let app_state = &all_state.app_state;
    let token = match main_token {
        Some(token) => token,
        None => {
            return Err(ErrorMessage::TokenNotProvided);
        }
    };

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
