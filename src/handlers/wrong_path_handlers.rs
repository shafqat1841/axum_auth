use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub async fn wrong_path() -> Response {
    (StatusCode::NOT_FOUND, "No path like this exist").into_response()
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;

    use super::*;

    #[tokio::test]
    async fn wrong_path_fn_body_test() {
        let result = wrong_path().await;
        assert_eq!(result.status(), 404);

        let body_bytes = to_bytes(result.into_body(), 1024).await.unwrap();

        // Convert bytes to a UTF-8 string
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

        assert_eq!(body_string, "No path like this exist");
    }
}
