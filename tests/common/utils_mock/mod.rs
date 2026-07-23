use std::{collections::HashMap, sync::Arc};

use anyhow::{Result, anyhow};
use axum::Router;
use axum_auth_v2::{AllStates, AppState, config::Config, db::DBClientMock, router::create_routes};
use tokio::sync::Mutex;

// use crate::common::db_mock::DBClientMock;
use axum_auth_v2::config::ConfigMockExt;


pub async fn create_mock_state() -> Result<AllStates<DBClientMock>> {
    let dummy_config = Config::mock()?;

    let dummy_db = Arc::new(DBClientMock::mock());

    let app_state = Arc::new(AppState {
        env: dummy_config,
        db_client: dummy_db,
    });

    let all_states = AllStates {
        app_state,
        refresh_tokens: Arc::new(Mutex::new(HashMap::new())),
    };

    Ok(all_states)
}

pub async fn get_app_mock() -> Router {
    let mock_state = create_mock_state()
        .await
        .map_err(|e| {
            let text = anyhow!("Error: {e}");
            panic!("{:?}", text);
        })
        .unwrap();
    let app_mock = create_routes(mock_state);
    app_mock
}