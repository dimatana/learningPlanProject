use crate::error::BetError;

#[derive(Debug, Clone, Copy)]
pub struct Stake(f64);

#[derive(Debug, Clone, Copy)]
pub struct Odds(f64);

#[derive(Debug, Clone, Copy)]
pub struct Id(u32);

#[derive(Debug, Clone, Copy)]
pub struct EventId(u32);

#[derive(Debug, Clone, Copy)]
pub struct Sum(f64);

impl Stake {
    pub fn new (value: f64) -> Result<Self, BetError> {
        if value <= 0.0 {
            return Err(BetError::InvalidStake(value));
        }
        Ok(Stake(value))
    }

    pub fn amount(self) -> f64{self.0}
}

impl Odds{
    pub fn new (value: f64) -> Result<Self, BetError> {
        if value <= 1.0 {
            return Err(BetError::InvalidOdds(value));
        }
        Ok(Odds(value))
    }
    pub fn value(self) -> f64{self.0}
}
impl EventId {
    pub fn new (value: u32) -> Result<Self, BetError> {
        if value < 1 {
            return Err(BetError::InvalidEventId(value));
        }
        Ok(EventId(value))
    }
    
}

#[derive(Debug)]
pub struct Bet{
    pub stake: Stake,
    pub odds: Odds,
    pub id: u32,
    pub event_id: u32,
    pub sum: f64,

}

impl Bet{
    pub fn potential_win(&self) -> f64 {
        self.stake.amount() * self.odds.value()
    }
    pub fn profit(&self) -> f64 {
        self.potential_win() - self.stake.amount()

    }
}

