use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error")]
    DatabaseError,

    #[error("invalid stake: {0}")]
    InvalidStake(f64),

    #[error("invalid odds: {0}")]
    InvalidOdds(f64),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::InvalidStake(v) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Invalid stake: {v}"),
            ),
            AppError::InvalidOdds(v) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Invalid odds: {v}"),
            ),
            AppError::DatabaseError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("Internal server error"),
            ),
        };

        let body = Json(serde_json::json!({ "error": message }));
        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    #[test]
    fn invalidd_stake_maps_to_422() {
        let response = AppError::InvalidStake(0.0).into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn invalid_odds_maps_to_422() {
        let response = AppError::InvalidOdds(0.5).into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn db_error_maps_to_500() {
        let response = AppError::DatabaseError.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
