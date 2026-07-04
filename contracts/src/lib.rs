//!shared events between 'bet_api' and 'trading-service' published/consumed through kafka

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event emitted by `bet_api` on the `bets.placed` topic after a bet
/// has been successfully saved to the database.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BetPlaced {
    ///new created bet id
    pub bet_id: Uuid,
    ///event id for placed bet
    pub event_id: Uuid,
    ///stake for the bet
    pub stake: f64,
    ///odds for the bet
    pub odds: f64,
    ///time when the bet was placed
    pub occured_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn bet_placed_serde_roundtrip() {
        let original = BetPlaced {
            bet_id: Uuid::new_v4(),
            event_id: Uuid::new_v4(),
            stake: 42.5,
            odds: 1.8,
            occured_at: Utc::now(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let decoded: BetPlaced = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, original);
    }
}
