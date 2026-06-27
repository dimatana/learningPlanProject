#[derive(Debug, Clone)]
struct Bet {
    id: u32,
    event: String,
    stake: f64,
    odds: f64,
}

struct AppState {
    bets: Vec<Bet>,
    total_staked: f64,
}


use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};
use tracing_subscriber::EnvFilter;

async fn place_bet(state: Arc<Mutex<AppState>>, bet: Bet) {
    debug!(id = bet.id, event = %bet.event, "Attempting to place bet");

    if bet.odds <= 1.0 {
        warn!(id = bet.id, odds = bet.odds, "Rejected bet with invalid odds");
        return;
    }

    if bet.stake <= 0.0 {
        warn!(id = bet.id, stake = bet.stake, "Rejected bet with non-positive stake");
        return;
    }

    let mut s = state.lock().await;
    s.total_staked += bet.stake;
    s.bets.push(bet.clone());

    info!(
        id = bet.id,
        event = %bet.event,
        stake = bet.stake,
        odds = bet.odds,
        total_staked = s.total_staked,
        "Bet placed successfully"
    );
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("BettingApp starting");

    let state = Arc::new(Mutex::new(AppState {
        bets: vec![],
        total_staked: 0.0,
    }));
    let s1 = Arc::clone(&state);
    let s2 = Arc::clone(&state);
    let s3 = Arc::clone(&state);
    let s4 = Arc::clone(&state);

    let t1 = tokio::spawn(place_bet(s1, Bet { id: 1, event: "Chelsea vs Arsenal".into(), stake: 50.0, odds: 2.10 }));
    let t2 = tokio::spawn(place_bet(s2, Bet { id: 2, event: "Man City vs Liverpool".into(), stake: 100.0, odds: 1.85 }));
    let t3 = tokio::spawn(place_bet(s3, Bet { id: 3, event: "Fake bet".into(), stake: 0.0, odds: 3.00 }));
    let t4 = tokio::spawn(place_bet(s4, Bet { id: 4, event: "Scam odds".into(), stake: 20.0, odds: 0.50 }));

    let _ = tokio::join!(t1, t2, t3, t4);

    print_summary(state).await;
}
async fn print_summary(state: Arc<Mutex<AppState>>) {
    let s = state.lock().await;

    info!(
        bet_count = s.bets.len(),
        total_staked = s.total_staked,
        "=== Betting summary ==="
    );

    for bet in &s.bets {
        debug!(
            id = bet.id,
            event = %bet.event,
            stake = bet.stake,
            odds = bet.odds,
            "Active bet"
        );
    }
}