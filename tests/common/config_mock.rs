use anyhow::Result;
use axum_auth_v2::config::Config;


pub trait ConfigMockExt {
    fn mock() -> Result<Config>;
}

impl ConfigMockExt for Config {
    fn mock() -> Result<Config> {
        dotenv::from_filename(".env.test").ok();
        Config::parse_env()
    }
}
