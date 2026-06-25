use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Bet {
    pub id: Uuid,
    pub event_id: Uuid,
    pub stake: f64,
    pub odds: f64,
    pub created_at: DateTime<Utc>,
}
