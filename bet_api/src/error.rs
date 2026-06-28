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
