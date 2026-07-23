#[cfg(test)]
mod tests {
    use axum::body::to_bytes;

    use crate::handlers::authorized_handlers::authorized_persons_only_handler::authorized_persons_only;

    #[tokio::test]
    async fn home_fn_body_test() {
        let result = authorized_persons_only().await;
        assert_eq!(result.status(), 200);

        let body_bytes = to_bytes(result.into_body(), 1024).await.unwrap();

        // Convert bytes to a UTF-8 string
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

        assert_eq!(body_string, "authorized Persons only");
    }
}
