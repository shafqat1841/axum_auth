use anyhow::{Context, Result};

pub trait ConfigTrait {
    fn init() -> Result<Config>;
}
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
    // Private helper to avoid code duplication
    pub fn parse_env() -> Result<Config> {
        let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
        let jwt_secret = std::env::var("JWT_SECRET_KEY").context("JWT_SECRET_KEY must be set")?;
        let jwt_maxage = std::env::var("JWT_MAXAGE")?
            .parse::<i64>()
            .context("Invalid JWT_MAXAGE")?;

        let refresh_jwt_secret = std::env::var("REFRESH_JWT_SECRET_KEY")
            .context("REFRESH_JWT_SECRET_KEY must be set")?;
        let refresh_jwt_maxage = std::env::var("REFRESH_JWT_MAXAGE")?
            .parse::<i64>()
            .context("Invalid REFRESH_JWT_MAXAGE")?;

        Ok(Config {
            database_url,
            jwt_secret,
            jwt_maxage,
            refresh_jwt_secret,
            refresh_jwt_maxage,
            port: 8000,
        })
    }
}

impl ConfigTrait for Config {
    fn init() -> Result<Config> {
        Self::parse_env()
    }
}
