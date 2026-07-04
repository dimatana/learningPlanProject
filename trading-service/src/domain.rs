use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A sporting event, with the number of bets placed on it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_id: Uuid,
    pub name: String,
    pub bets_placed: i64,
}
