mod config;
mod database;
mod db;
mod dtos;
mod errors;
mod handlers;
mod middlewares;
mod models;
mod router;
mod utils;

use axum::http::{
    HeaderValue, Method,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use dotenv::dotenv;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::{config::Config, db::DBClient};

#[derive(Debug, Clone)]
pub struct AppState {
    pub env: Config,
    pub db_client: DBClient,
}

async fn get_database_pool(config: &Config) -> Pool<Postgres> {
    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url.clone())
        .await
    {
        Ok(pool) => {
            println!("✅Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("🔥 Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    pool
}

fn setup_cors() -> CorsLayer {
    let cors = CorsLayer::new()
        .allow_origin("https://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT]);
    cors
}

fn get_app_state(configuration: &Config, pool: Pool<Postgres>) -> AppState {
    let db_client = DBClient::new(pool);
    AppState {
        env: configuration.clone(),
        db_client,
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let configuration = Config::init();

    let pool = get_database_pool(&configuration).await;

    let cors = setup_cors();

    let app_state = get_app_state(&configuration, pool);


    // build our application with a single route
    let app_api = router::create_routes(Arc::new(app_state)).layer(cors);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app_api).await.unwrap();
}
