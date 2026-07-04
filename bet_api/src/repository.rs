use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// A bet as stored in the database.
#[derive(Serialize, Deserialize, Debug)]
pub struct Bet {
    pub id: Uuid,
    pub event_id: Uuid,
    pub stake: f64,
    pub odds: f64,
    pub created_at: DateTime<Utc>,
}
/// Insert a new bet and return the created row (with `id`/`created_at` generated).
pub async fn insert_bet(
    pool: &PgPool,
    event_id: Uuid,
    stake: f64,
    odds: f64,
) -> Result<Bet, sqlx::Error> {
    sqlx::query_as!(
        Bet,
        "INSERT INTO bets (event_id, stake, odds) VALUES ($1, $2, $3)
         RETURNING id, event_id, stake, odds, created_at",
        event_id,
        stake,
        odds
    )
    .fetch_one(pool)
    .await
}
/// Search a bet by id. Returns 'None' if not found.
pub async fn fetch_bet_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Bet>, sqlx::Error> {
    sqlx::query_as!(
        Bet,
        "SELECT id, event_id, stake, odds, created_at FROM bets WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await
}
/// List all bets, most recent first.
pub async fn list_bets(pool: &PgPool) -> Result<Vec<Bet>, sqlx::Error> {
    sqlx::query_as!(
        Bet,
        "SELECT id, event_id, stake, odds, created_at FROM bets ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await
}
