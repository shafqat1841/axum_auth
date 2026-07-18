use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_maxage: i64,
    pub refresh_jwt_secret: String,
    pub refresh_jwt_maxage: i64,
    pub port: u16,
}

impl Config {
    pub fn init() -> Result<Config> {
        let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
        let jwt_secret = std::env::var("JWT_SECRET_KEY").context("JWT_SECRET_KEY must be set")?;
        let jwt_maxage = std::env::var("JWT_MAXAGE").context("JWT_MAXAGE must be set")?;

        let refresh_jwt_secret =
            std::env::var("REFRESH_JWT_SECRET_KEY").context("REFRESH_JWT_SECRET_KEY must be set")?;
        let refresh_jwt_maxage =
            std::env::var("REFRESH_JWT_MAXAGE").context("REFRESH_JWT_MAXAGE must be set")?;

        Ok(Config {
            database_url,
            jwt_secret,
            jwt_maxage: jwt_maxage.parse::<i64>().unwrap(),
            refresh_jwt_secret,
            refresh_jwt_maxage: refresh_jwt_maxage.parse::<i64>().unwrap(),
            port: 8000,
        })
    }
}
