use service_common::parse_u32_env;

/// `bet_api` runtime configuration, read from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub kafka_bootstrap_servers: String,
    pub bind_addr: String,
    pub database_min_connections: u32,
    pub database_max_connections: u32,
}

impl Config {
    /// Reads configuration from environment variables, applying default
    /// values where it makes sense (see each field below).
    ///
    /// # Panics
    /// Panics if `DATABASE_URL` is missing — the service cannot start without it.
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let kafka_bootstrap_servers = std::env::var("KAFKA_BOOTSTRAP_SERVERS")
            .unwrap_or_else(|_| "localhost:19092".to_string());

        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

        let database_min_connections =
            parse_u32_env(std::env::var("DATABASE_MIN_CONNECTIONS").ok().as_deref(), 1);
        let database_max_connections =
            parse_u32_env(std::env::var("DATABASE_MAX_CONNECTIONS").ok().as_deref(), 5);

        Self {
            database_url,
            kafka_bootstrap_servers,
            bind_addr,
            database_min_connections,
            database_max_connections,
        }
    }
}
