use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub async fn home() -> Response {
    (StatusCode::OK, "Home route").into_response()
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;

    use super::*;

    #[tokio::test]
    async fn home_fn_body_test() {
        let result = home().await;
        assert_eq!(result.status(), 200);

        let body_bytes = to_bytes(result.into_body(), 1024).await.unwrap();

        // Convert bytes to a UTF-8 string
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

        assert_eq!(body_string, "Home route");
    }
}
