// service-common/src/env.rs

/// Parses an optional environment variable as `u32`, returning `default`
/// if the variable is missing or its contents are not a valid `u32`.
pub fn parse_u32_env(raw: Option<&str>, default: u32) -> u32 {
    raw.and_then(|v| v.parse::<u32>().ok()).unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_value_is_used() {
        assert_eq!(parse_u32_env(Some("7"), 1), 7);
    }

    #[test]
    fn missing_value_falls_back_to_default() {
        assert_eq!(parse_u32_env(None, 5), 5);
    }

    #[test]
    fn non_numeric_value_falls_back_to_default() {
        assert_eq!(parse_u32_env(Some("not-a-number"), 5), 5);
    }

    #[test]
    fn negative_value_falls_back_to_default() {
        assert_eq!(parse_u32_env(Some("-3"), 5), 5);
    }

    #[test]
    fn empty_string_falls_back_to_default() {
        assert_eq!(parse_u32_env(Some(""), 5), 5);
    }
}
