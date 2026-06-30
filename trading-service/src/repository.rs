use crate::domain::Event;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn fetch_event_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Event>, sqlx::Error> {
    sqlx::query_as!(
        Event,
        "SELECT event_id, name, bets_placed FROM events WHERE event_id = $1",
        id
    )
        .fetch_optional(pool)
        .await
}