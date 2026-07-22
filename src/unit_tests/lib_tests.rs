#[cfg(test)]
mod tests {
    use crate::{config::Config, get_all_states, get_app_state, setup_cors};

    fn get_mock_config() -> Config {
        // Create a mock/dummy config
        let config = Config {
            database_url: "postgres://user:pass@localhost:5432/db".to_string(),
            jwt_secret: "secret".to_string(),
            jwt_maxage: 3600,
            refresh_jwt_secret: "refresh_secret".to_string(),
            refresh_jwt_maxage: 7200,
            port: 3000,
        };

        config
    }

    #[test]
    fn test_setup_cors_returns_valid_layer() {
        // Direct test for your setup_cors function
        let cors = setup_cors();

        // Ensure the cors layer can be successfully applied to an Axum router
        let _router: axum::Router<()> = axum::Router::new().layer(cors);
    }

   #[tokio::test]
    async fn test_get_app_state_initialization() {

        let config = get_mock_config();

        // Create a dummy pool using sqlx offline/connect_lazy (doesn't require a live database connection)
        let pool = sqlx::PgPool::connect_lazy("postgres://user:pass@localhost:5432/db").unwrap();

        // Directly test your get_app_state function
        let app_state = get_app_state(&config, pool);

        assert_eq!(app_state.env.port, 3000);
        assert_eq!(app_state.env.jwt_secret, "secret");
    }

    #[tokio::test]
    async fn test_get_all_states_initialization() {
        // Create a mock/dummy config
        let config = get_mock_config();

        let pool = sqlx::PgPool::connect_lazy("postgres://user:pass@localhost:5432/db").unwrap();

        // Directly test your get_all_states function
        let all_states = get_all_states(&config, pool);

        // Verify the app state is properly nested inside all_states
        assert_eq!(all_states.app_state.env.port, 3000);

        // Verify the refresh tokens map starts completely empty
        let blocking_check =  all_states.refresh_tokens.lock().await.len();
        assert_eq!(blocking_check, 0);
    }
}
