use crate::domain::Bet;
use sqlx::PgPool;
use uuid::Uuid;

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

pub async fn fetch_bet_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Bet>, sqlx::Error> {
    sqlx::query_as!(
        Bet,
        "SELECT id, event_id, stake, odds, created_at FROM bets WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await
}
