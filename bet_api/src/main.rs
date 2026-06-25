mod config;
mod docs;
mod domain;
mod error;
mod repository;
mod state;

use crate::config::Config;
use crate::domain::Bet;
use crate::error::AppError;
use crate::state::AppState;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;
use axum::{Router, routing::get};
use chrono::Utc;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use sqlx::postgres::PgPoolOptions;
use tower_http::trace::TraceLayer;
use tracing_subscriber;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let config = Config::from_env();

    let pool = PgPoolOptions::new()
        .max_connections(5)
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

    let app = Router::new()
        .route("/health", get(health))
        .route("/bets", get(list_bet).post(create_bet))
        .route("/bets/:id", get(get_bet))
        .merge(docs::router())
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
async fn health() -> &'static str {
    "ok"
}

async fn create_bet(
    State(state): State<AppState>,
    Json(payload): Json<bet_api_generated::models::PlaceBetRequest>,
) -> Result<Json<Bet>, AppError> {
    let bet = repository::insert_bet(&state.pool, payload.event_id, payload.odds, payload.stake)
        .await
        .map_err(|_| AppError::DatabaseError)?;

    let event = contracts::BetPlaced {
        bet_id: bet.id,
        event_id: bet.event_id,
        stake: bet.stake,
        odds: bet.odds,
        occured_at: Utc::now(),
    };

    let event_payload = serde_json::to_string(&event).unwrap();

    let delivery = state
        .producer
        .send(
            FutureRecord::to("bets-events")
                .key(&bet.id.to_string())
                .payload(&event_payload),
            std::time::Duration::from_secs(5),
        )
        .await;

    if let Err((e, _)) = delivery {
        tracing::error!("failed to publish event to redpanda: {:?}", e);
    }

    Ok(Json(bet))
}

async fn get_bet(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Bet>, AppError> {
    let bet = repository::fetch_bet_by_id(&state.pool, id)
        .await
        .map_err(|_| AppError::DatabaseError)?
        .ok_or(AppError::NotFound(id))?;

    Ok(Json(bet))
}

async fn list_bet(State(state): State<AppState>) -> Result<Json<Vec<Bet>>, AppError> {
    let bets = sqlx::query_as!(
        Bet,
        "SELECT id, event_id, stake, odds, created_at FROM bets ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    Ok(Json(bets))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn not_found_returns_404() {
        let id = Uuid::new_v4();
        let error = AppError::NotFound(id);
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("not found"));
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
    async fn bet_error_status_codes() {
        let id = Uuid::new_v4();

        let cases = vec![
            (AppError::NotFound(id), StatusCode::NOT_FOUND),
            (AppError::DatabaseError, StatusCode::INTERNAL_SERVER_ERROR),
        ];

        for (error, expected_status) in cases {
            let response = error.into_response();
            assert_eq!(response.status(), expected_status);
        }
    }
}
