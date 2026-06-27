use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use bet_api_generated::models::PlaceBetRequest;
use crate::error::AppError;

#[derive(Serialize, Deserialize, Debug)]
pub struct Bet {
    pub id: Uuid,
    pub event_id: Uuid,
    pub stake: f64,
    pub odds: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy)]
pub struct ValidBet {
    pub event_id: Uuid,
    pub stake: f64,
    pub odds: f64,
}

pub fn validate_place_bet(req: PlaceBetRequest) -> Result<ValidBet, AppError> {
    if req.stake <= 0.0 {
        return Err(AppError::InvalidStake(req.stake));
    }

    // Cerinta: odds < 1.0 este invalid (1.0 ramane valid)
    if req.odds < 1.0 {
        return Err(AppError::InvalidOdds(req.odds));
    }

    Ok(ValidBet {
        event_id: req.event_id,
        stake: req.stake,
        odds: req.odds,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bet_api_generated::models::PlaceBetRequest;
    use uuid::Uuid;

    fn req(stake: f64, odds: f64) -> PlaceBetRequest {
        PlaceBetRequest {
            event_id: Uuid::new_v4(),
            stake,
            odds,
        }
    }

    #[test]
    fn happy_case() {
        let result = validate_place_bet(req(50.0, 2.0));
        assert!(result.is_ok());
    }

    #[test]
    fn zero_stake_is_rejected() {
        let result = validate_place_bet(req(0.0, 2.0));
        assert!(matches!(result, Err(AppError::InvalidStake(_))));
    }

    #[test]
    fn negative_stake_is_rejected() {
        let result = validate_place_bet(req(-10.0, 2.0));
        assert!(matches!(result, Err(AppError::InvalidStake(_))));
    }

    #[test]
    fn odds_below_1_are_rejected() {
        let result = validate_place_bet(req(50.0, 0.99));
        assert!(matches!(result, Err(AppError::InvalidOdds(_))));
    }

    #[test]
    fn odds_equal_1_is_valid() {
        let result = validate_place_bet(req(50.0, 1.0));
        assert!(result.is_ok());
    }
}