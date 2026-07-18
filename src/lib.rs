pub mod config;
pub mod database;
pub mod db;
pub mod dtos;
pub mod errors;
pub mod handlers;
pub mod middlewares;
pub mod models;
pub mod router;
pub mod utils;

use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result, anyhow};
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

pub async fn get_database_pool(config: &Config) -> Result<Pool<Postgres>> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url.clone())
        .await
        .map_err(|e| anyhow!("Failed to connect to the database: {e}"))?;

    Ok(pool)
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

pub async fn config_all_and_get_all_requirments() -> Result<(AllStates, CorsLayer)> {
    let configuration = Config::init().context("Error making config")?;

    let pool = get_database_pool(&configuration)
        .await
        .map_err(|e| anyhow!("Fail to get database pool: {e}"))?;

    let cors = setup_cors();

    let all_state = get_all_states(&configuration, pool);
    Ok((all_state, cors))
}

pub async fn app() {
    dotenv().ok();

    let config_all_res = config_all_and_get_all_requirments().await;

    let (all_states, cors) = match config_all_res {
        Ok(res) => res,
        Err(err) => {
            let error = anyhow!("Error from config all: {err}");
            eprint!("{}", error);
            std::process::exit(1);
        }
    };

    // build our application with a single route
    let app_api = router::create_routes(all_states).layer(cors);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await;

    let listener = match listener {
        Err(e) => {
            let err = anyhow!("Error at TcpListener binding {e}");
            eprintln!("{}", err);
            std::process::exit(1);
        }
        Ok(listener) => {
            println!("TcpListener connected");
            listener
        }
    };

    if let Err(e) = axum::serve(listener, app_api).await {
        let err = anyhow!("Error at serving service: {e}");
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
