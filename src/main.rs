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

use std::{
    collections::HashMap,
    sync::{Arc},
};

use dotenv::dotenv;

use axum::http::{
    HeaderValue, Method,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::{config::Config, db::DBClient};


#[derive(Debug, Clone)]
pub struct AppState {
    pub env: Config,
    pub db_client: DBClient,
}

#[derive(Clone)]
pub struct AllStates {
    pub app_state: Arc<AppState>,
    pub refresh_tokens: Arc<Mutex<HashMap<String, String>>>,
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

fn get_all_states(configuration: &Config, pool: Pool<Postgres>) -> AllStates {
    let app_state = Arc::new(get_app_state(configuration, pool));
    let refresh_tokens = Arc::new(Mutex::new(HashMap::new()));
    AllStates {
        app_state,
        refresh_tokens,
    }

}

pub async fn config_all_and_get_all_requirments() -> (AllStates, CorsLayer) {

    let configuration = Config::init();

    let pool = get_database_pool(&configuration).await;

    let cors = setup_cors();

    let all_state = get_all_states(&configuration, pool);
    (all_state, cors)
}


#[tokio::main]
async fn main() {
    dotenv().ok();

    let (all_states, cors) = config_all_and_get_all_requirments().await;

    // build our application with a single route
    let app_api = router::create_routes(all_states).layer(cors);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app_api).await.unwrap();
}
