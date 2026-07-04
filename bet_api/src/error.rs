use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
/// Possible errors in an HTTP handler of `bet_api`.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// A database operation failed. Keep the source error for logs,
    /// but do not expose it to the client (generic message to the consumer).
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// The bet amount does not comply with the business rules (ex: <= 0).
    #[error("invalid stake: {0}")]
    InvalidStake(f64),

    /// The odds value does not comply with the business rules (ex: <= 0).
    #[error("invalid odds: {0}")]
    InvalidOdds(f64),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if let AppError::Database(ref e) = self {
            tracing::error!(error = %e, "database operation failed");
        }
        let (status, message) = match &self {
            AppError::InvalidStake(_) | AppError::InvalidOdds(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, self.to_string())
            }
            AppError::Database(_) => (
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
        let response = AppError::Database(sqlx::Error::RowNotFound).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
