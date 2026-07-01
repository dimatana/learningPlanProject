mod api_impl;
mod config;
mod docs;
mod domain;
mod error;
mod repository;
mod state;

use crate::api_impl::ApiImpl;
use crate::config::Config;
use crate::state::AppState;
use rdkafka::config::ClientConfig;
use rdkafka::producer::FutureProducer;
use sqlx::postgres::PgPoolOptions;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();

    let pool = PgPoolOptions::new()
        .min_connections(config.database_min_connections)
        .max_connections(config.database_max_connections)
        .connect(&config.database_url)
        .await
        .expect("failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &config.kafka_bootstrap_servers)
        .create()
        .expect("failed to create producer");

    let state = AppState { pool, producer };
    let api_impl = ApiImpl::new(state);

    let app = bet_api_generated::server::new(api_impl)
        .merge(docs::router())
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .expect("failed to bind");

    info!(bind_addr = %config.bind_addr, "HTTP server started");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install ctrl+c handler")
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => warn!("received Ctrl-C, shutting down"),
        _ = terminate => warn!("received SIGTERM, shutting down"),
    }
}

#[cfg(test)]
mod tests {
    use crate::error::AppError;
    use axum::body::to_bytes;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn generated_not_found_payload_shape_is_valid() {
        let body = bet_api_generated::models::ErrorResponse {
            error: "Bet not found".to_string(),
        };
        let response = (StatusCode::NOT_FOUND, axum::Json(body)).into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(body_str.contains("Bet not found"));
    }

    #[tokio::test]
    async fn database_error_returns_500() {
        let error = AppError::DatabaseError;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("Internal server error"));
    }

    #[tokio::test]
    async fn invalid_stake_returns_422() {
        let error = AppError::InvalidStake(0.0);
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn invalid_odds_returns_422() {
        let error = AppError::InvalidOdds(0.5);
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
