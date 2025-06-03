use serde::Deserialize;
use std::env;

/// Application configuration (e.g. JWT secret, database URL, etc.)
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    /// JWT secret used for signing/verifying tokens
    pub jwt_secret: String,
    /// Database URL (e.g., Postgres connection string)
    pub database_url: String,
    // Add more fields as needed (e.g. server host, port, etc.)
}

impl Config {
    /// Load configuration from environment variables (using `envy` or plain `env::var`)
    // TODO do I need envy?
    pub fn load() -> Result<Self, envy::Error> {
        envy::from_env::<Config>()
    }
}

impl Default for Config {
    fn default() -> Self {
        // TODO maybe get rid of this class
        Self {
            jwt_secret: env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set"),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        }
    }
}
