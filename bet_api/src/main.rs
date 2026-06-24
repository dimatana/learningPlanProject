mod docs;

use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;
use tracing_subscriber;
use axum::extract::Path;
use uuid::Uuid;
use axum::Json;
use axum::http::StatusCode;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use chrono::{DateTime, Utc};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to database");

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:19092")
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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
async fn health() -> &'static str {
    "ok"
}




#[derive(Deserialize)]
struct CreateBet {
    stake: f64,
    odds: f64,
    bookmaker: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Bet {
    id: Uuid,
    stake: f64,
    odds: f64,
    bookmaker: String,
    status: String,
    created_at: DateTime<Utc>,
}

async fn create_bet(State(state) : State<AppState>,
                    Json(payload): Json<CreateBet>,
               ) -> Result<Json<Bet>, BetError> {
    let bet = sqlx::query_as!(
         Bet,
        "INSERT INTO bets (stake, odds, bookmaker) VALUES ($1, $2, $3) RETURNING *",

        payload.stake,
        payload.odds,
        payload.bookmaker
    )
        .fetch_one(&state.pool)
        .await
        .map_err(|_| BetError::DatabaseError)?;

    let event_payload = serde_json::to_string(&bet).unwrap();

    let delivery = state.producer
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

async fn get_bet(State(state): State<AppState>,
                 Path(id): Path<Uuid>,
            ) -> Result<Json<Bet>, BetError> {
    let bet = sqlx::query_as! (
        Bet,
        "SELECT * FROM bets WHERE id = $1",
        id
    )
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| BetError::DatabaseError)?
        .ok_or(BetError::NotFound(id))?;

    Ok(Json(bet))
}

#[derive(Clone)]
struct AppState {
    pool: sqlx::PgPool,
    producer: FutureProducer,
}
async fn list_bet(State(state): State<AppState>) -> Result <Json<Vec<Bet>>,BetError> {
    let bets = sqlx::query_as!(
        Bet,
        "SELECT * FROM bets ORDER BY created_at DESC"
    )
    .fetch_all(&state.pool)
    .await
        .map_err(|_| BetError::DatabaseError)?;

    Ok(Json(bets))
}



impl IntoResponse for BetError {
    fn into_response(self) -> Response {
        let(status, message) = match self {
            BetError::NotFound(id) => (StatusCode::NOT_FOUND, format!("Bet {id} not found")),
            BetError::DatabaseError => (StatusCode::INTERNAL_SERVER_ERROR, String::from("Internal server error")),
        };
        let body = Json(serde_json::json!({"error": message}));
        (status, body).into_response()
    }
}


#[derive(Debug, thiserror::Error)]
enum BetError {
    #[error("bet not found")]
    NotFound(Uuid),
    #[error("database error")]
    DatabaseError,
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    #[tokio::test]
    async fn not_found_returns_404() {
        let id = Uuid::new_v4();
        let error = BetError::NotFound(id);
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("not found"));
    }

    #[tokio::test]
    async fn database_error_returns_500() {
        let error = BetError::DatabaseError;
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
            (BetError::NotFound(id), StatusCode::NOT_FOUND),
            (BetError::DatabaseError, StatusCode::INTERNAL_SERVER_ERROR),
        ];

        for(error, expected_status) in cases {
            let response = error.into_response();
            assert_eq!(response.status(), expected_status);
        }
    }
}