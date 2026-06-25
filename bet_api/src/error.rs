use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("bet not found")]
    NotFound(Uuid),

    #[error("database error")]
    DatabaseError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(id) => (StatusCode::NOT_FOUND, format!("Bet {id} not found")),
            AppError::DatabaseError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("Internal server error"),
            ),
        };

        let body = Json(serde_json::json!({ "error": message }));
        (status, body).into_response()
    }
}
