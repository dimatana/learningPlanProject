use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BetPlaced {
    pub bet_id: Uuid,
    pub event_id: Uuid,
    pub stake: f64,
    pub odds: f64,
    pub occured_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

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
