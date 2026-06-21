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


    let app = Router::new().route("/health", get(health))
        .route("/bets", get(list_bet).post(create_bet))
        .route("/bets/:id", get(get_bet))
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

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
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
    let bet = sqlx::query_as::<_, Bet>(
        "INSERT INTO bets (stake, odds, bookmaker) VALUES ($1, $2, $3) RETURNING *"
    )
        .bind(payload.stake)
        .bind(payload.odds)
        .bind(payload.bookmaker)
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
    let bet = sqlx::query_as::<_, Bet>("SELECT * FROM bets WHERE id = $1")
        .bind(id)
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
    let bets = sqlx::query_as::<_, Bet>("SELECT * FROM bets ORDER BY created_at DESC")
    .fetch_all(&state.pool)
    .await
        .map_err(|_| BetError::DatabaseError)?;

    Ok(Json(bets))
}



impl IntoResponse for BetError {
    fn into_response(self) -> Response {
        let(status, message) = match self {
            BetError::NotFound(id) => (StatusCode::NOT_FOUND, format!("Bet {id} not_found")),
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