#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub kafka_bootstrap_servers: String,
    pub bind_addr: String,
    pub database_min_connections: u32,
    pub database_max_connections: u32,
}

impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let kafka_bootstrap_servers = std::env::var("KAFKA_BOOTSTRAP_SERVERS")
            .unwrap_or_else(|_| "localhost:19092".to_string());

        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

        let database_min_connections = parse_u32_env("DATABASE_MIN_CONNECTIONS", 1);
        let database_max_connections = parse_u32_env("DATABASE_MAX_CONNECTIONS", 5);

        Self {
            database_url,
            kafka_bootstrap_servers,
            bind_addr,
            database_min_connections,
            database_max_connections,
        }
    }
}
fn parse_u32_env(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(default)
}
