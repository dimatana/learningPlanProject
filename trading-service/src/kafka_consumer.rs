use contracts::BetPlaced;
use futures::StreamExt;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message;
use sqlx::PgPool;
use tracing::{error, info, warn};

use crate::repository;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    EmptyPayload,
    InvalidJson(String),
}

pub fn decode_message(bytes: &[u8]) -> Result<BetPlaced, SkipReason> {
    if bytes.is_empty() {
        return Err(SkipReason::EmptyPayload);
    }

    serde_json::from_slice::<BetPlaced>(bytes)
        .map_err(|e| SkipReason::InvalidJson(e.to_string()))
}

pub async fn run(pool: PgPool, brokers: String) -> anyhow::Result<()> {
    let consumer: StreamConsumer = rdkafka::ClientConfig::new()
        .set("bootstrap.servers", &brokers)
        .set("group.id", "trading-service-v1")
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .create()?;

    consumer.subscribe(&["bets.placed"])?;
    info!("Kafka consumer subscribed to bets.placed");

    let mut stream = consumer.stream();

    while let Some(message_result) = stream.next().await {
        let message = match message_result {
            Ok(msg) => msg,
            Err(e) => {
                warn!(error = %e, "kafka poll error; skipping message");
                continue;
            }
        };

        let payload = message.payload().unwrap_or(&[]);

        match decode_message(payload) {
            Ok(event) => {
                match repository::mark_bet_processed(&pool, event.bet_id).await {
                    Ok(true) => {
                        match repository::increment_bets_placed(&pool, event.event_id).await {
                            Ok(1) => {
                                info!(event_id = %event.event_id, bet_id = %event.bet_id, "incremented bets_placed");
                            }
                            Ok(0) => {
                                warn!(event_id = %event.event_id, "unknown event id; skipping");
                            }
                            Ok(rows) => {
                                warn!(rows_affected = rows, "unexpected number of rows affected");
                            }
                            Err(e) => {
                                error!(error = %e, "database update failed");
                            }
                        }
                    }
                Ok(false) => {
                    info!(bet_id = %event.bet_id, "bet already proccesed; skipping dublicate")
                }
                Err(e) =>{
                error!(error = %e, "failed to check processed bet");
                }
                }
            }
            Err(reason) => {
                warn!(?reason, "skipping malformed or empty kafka payload");
            }
        }

        if let Err(e) = consumer.commit_message(&message, CommitMode::Async) {
            warn!(error = %e, "offset commit failed");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn decode_valid_json() {
        let bytes = br#"{
            "bet_id":"aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
            "event_id":"11111111-1111-1111-1111-111111111111",
            "stake":10.5,
            "odds":1.8,
            "occured_at":"2026-06-30T19:00:00Z"
        }"#;

        let decoded = decode_message(bytes).expect("valid JSON should decode");
        assert_eq!(
            decoded.event_id,
            Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap()
        );
    }

    #[test]
    fn decode_malformed_json_returns_skip() {
        let bad = br#"{"event_id":"broken""#;
        let err = decode_message(bad).unwrap_err();

        match err {
            SkipReason::InvalidJson(_) => {}
            _ => panic!("expected InvalidJson"),
        }
    }

    #[test]
    fn decode_empty_payload_returns_skip() {
        let err = decode_message(&[]).unwrap_err();
        assert_eq!(err, SkipReason::EmptyPayload);
    }
}