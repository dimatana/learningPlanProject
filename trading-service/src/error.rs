use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Possible errors in an HTTP handler of `trading-service`.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let AppError::Database(ref e) = self;
        tracing::error!(error = %e, "database operation failed");

        let body = Json(serde_json::json!({ "error": "Internal server error" }));
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    #[tokio::test]
    async fn database_error_maps_to_500() {
        let response = AppError::Database(sqlx::Error::RowNotFound).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(body_str.contains("Internal server error"));
    }
}
