use crate::error::AppError;
use bet_api_generated::models::PlaceBetRequest;
use uuid::Uuid;

/// A bet that has already passed business validations and is ready to be saved.
#[derive(Debug, Clone, Copy)]
pub struct ValidBet {
    pub event_id: Uuid,
    pub stake: f64,
    pub odds: f64,
}
/// Validates the request to place a bet.
///
/// Rules: the amount must be strictly positive and the odds must be >= 1.0.
pub fn validate_place_bet(req: &PlaceBetRequest) -> Result<ValidBet, AppError> {
    if req.stake <= 0.0 {
        return Err(AppError::InvalidStake(req.stake));
    }

    // Requirement: odds < 1.0 is invalid (1.0 remains valid)
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
        let result = validate_place_bet(&req(50.0, 2.0));
        assert!(result.is_ok());
    }

    #[test]
    fn zero_stake_is_rejected() {
        let result = validate_place_bet(&req(0.0, 2.0));
        assert!(matches!(result, Err(AppError::InvalidStake(_))));
    }

    #[test]
    fn negative_stake_is_rejected() {
        let result = validate_place_bet(&req(-10.0, 2.0));
        assert!(matches!(result, Err(AppError::InvalidStake(_))));
    }

    #[test]
    fn odds_below_1_are_rejected() {
        let result = validate_place_bet(&req(50.0, 0.99));
        assert!(matches!(result, Err(AppError::InvalidOdds(_))));
    }

    #[test]
    fn odds_equal_1_is_valid() {
        let result = validate_place_bet(&req(50.0, 1.0));
        assert!(result.is_ok());
    }
}
