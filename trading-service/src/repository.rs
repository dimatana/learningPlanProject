use crate::domain::Event;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn fetch_event_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Event>, sqlx::Error> {
    sqlx::query_as!(
        Event,
        r#"SELECT event_id, name, bets_placed as "bets_placed!: i64" FROM events WHERE event_id = $1"#,
        id
    )
        .fetch_optional(pool)
        .await
}

pub async fn increment_bets_placed(pool: &PgPool, event_id: Uuid) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        r#"UPDATE events SET bets_placed = bets_placed + 1 WHERE event_id = $1"#,
        event_id
    )
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}