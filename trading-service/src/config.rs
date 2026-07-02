#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    pub database_min_connections: u32,
    pub database_max_connections: u32,
}

impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3001".to_string());
        let database_min_connections =
            parse_u32(std::env::var("DATABASE_MIN_CONNECTIONS").ok().as_deref(), 1);
        let database_max_connections =
            parse_u32(std::env::var("DATABASE_MAX_CONNECTIONS").ok().as_deref(), 5);

        Self {
            database_url,
            bind_addr,
            database_min_connections,
            database_max_connections,
        }
    }
}

fn parse_u32(raw: Option<&str>, default: u32) -> u32 {
    raw.and_then(|v| v.parse::<u32>().ok()).unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_u32_valid_value_is_used() {
        assert_eq!(parse_u32(Some("7"), 1), 7);
    }

    #[test]
    fn parse_u32_missing_value_falls_back_to_default() {
        assert_eq!(parse_u32(None, 5), 5);
    }

    #[test]
    fn parse_u32_non_numeric_value_falls_back_to_default() {
        assert_eq!(parse_u32(Some("not-a-number"), 5), 5);
    }

    #[test]
    fn parse_u32_negative_value_falls_back_to_default() {
        assert_eq!(parse_u32(Some("-3"), 5), 5);
    }

    #[test]
    fn parse_u32_empty_string_falls_back_to_default() {
        assert_eq!(parse_u32(Some(""), 5), 5);
    }
}
