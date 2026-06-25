use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetPlaced {
    pub bet_id: Uuid,
    pub event_id: Uuid,
    pub stake: f64,
    pub odds: f64,
    pub occured_at: DateTime<Utc>,
}
