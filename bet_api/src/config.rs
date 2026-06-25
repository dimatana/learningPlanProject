#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub kafka_bootstrap_servers: String,
    pub bind_addr: String,
}

impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let kafka_bootstrap_servers = std::env::var("KAFKA_BOOTSTRAP_SERVERS")
            .unwrap_or_else(|_| "localhost:19092".to_string());

        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

        Self {
            database_url,
            kafka_bootstrap_servers,
            bind_addr,
        }
    }
}
