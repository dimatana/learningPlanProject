use service_common::parse_u32_env;

/// The `trading-service` runtime configuration, read from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    pub database_min_connections: u32,
    pub database_max_connections: u32,
}

impl Config {
    /// # Panics
    /// Panic if `DATABASE_URL` is missing.
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3001".to_string());
        let database_min_connections =
            parse_u32_env(std::env::var("DATABASE_MIN_CONNECTIONS").ok().as_deref(), 1);
        let database_max_connections =
            parse_u32_env(std::env::var("DATABASE_MAX_CONNECTIONS").ok().as_deref(), 5);

        Self {
            database_url,
            bind_addr,
            database_min_connections,
            database_max_connections,
        }
    }
}
