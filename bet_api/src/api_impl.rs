use std::sync::Arc;

use async_trait::async_trait;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use bet_api_generated::apis;
use bet_api_generated::apis::default::{
    Default as DefaultApi, GetBetResponse, GetHealthResponse, ListBetsResponse, PlaceBetResponse,
};
use bet_api_generated::models;
use chrono::Utc;
use headers::Host;
use http::Method;
use rdkafka::producer::FutureRecord;

use crate::domain;
use crate::error::AppError;
use crate::repository::{self, Bet};
use crate::state::AppState;

/// Concrete implementation of routes generated from `openapi.yaml`.
#[derive(Clone)]
pub struct ApiImpl {
    pub state: Arc<AppState>,
}

impl ApiImpl {
    pub fn new(state: AppState) -> Self {
        Self {
            state: Arc::new(state),
        }
    }

    fn to_model_bet(b: Bet) -> models::Bet {
        models::Bet {
            id: b.id,
            event_id: b.event_id,
            stake: b.stake,
            odds: b.odds,
            created_at: b.created_at,
        }
    }

    fn err_body(msg: impl Into<String>) -> models::ErrorResponse {
        models::ErrorResponse { error: msg.into() }
    }
}

impl AsRef<ApiImpl> for ApiImpl {
    fn as_ref(&self) -> &ApiImpl {
        self
    }
}

#[async_trait]
impl apis::ErrorHandler<AppError> for ApiImpl {
    async fn handle_error(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        error: AppError,
    ) -> Result<axum::response::Response, http::StatusCode> {
        Ok(error.into_response())
    }
}

#[async_trait]
impl DefaultApi<AppError> for ApiImpl {
    async fn get_health(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
    ) -> Result<GetHealthResponse, AppError> {
        Ok(GetHealthResponse::Status200_Ok("ok".to_string()))
    }

    async fn list_bets(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
    ) -> Result<ListBetsResponse, AppError> {
        let bets = repository::list_bets(&self.state.pool).await?;

        let model_bets = bets.into_iter().map(Self::to_model_bet).collect::<Vec<_>>();
        Ok(ListBetsResponse::Status200_ListOfBets(model_bets))
    }

    async fn get_bet(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        path_params: &models::GetBetPathParams,
    ) -> Result<GetBetResponse, AppError> {
        let id = path_params.id;

        let bet = repository::fetch_bet_by_id(&self.state.pool, id).await?;

        match bet {
            Some(b) => Ok(GetBetResponse::Status200_BetFound(Self::to_model_bet(b))),
            None => Ok(GetBetResponse::Status404_BetNotFound(Self::err_body(
                format!("Bet {id} not found"),
            ))),
        }
    }

    async fn place_bet(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        body: &models::PlaceBetRequest,
    ) -> Result<PlaceBetResponse, AppError> {
        let valid = match domain::validate_place_bet(body) {
            Ok(v) => v,
            Err(e @ (AppError::InvalidStake(_) | AppError::InvalidOdds(_))) => {
                return Ok(PlaceBetResponse::Status422_InvalidRequest(Self::err_body(
                    e.to_string(),
                )));
            }
            Err(e) => return Err(e),
        };

        let bet = repository::insert_bet(&self.state.pool, valid.event_id, valid.stake, valid.odds)
            .await?;

        let event = contracts::BetPlaced {
            bet_id: bet.id,
            event_id: bet.event_id,
            stake: bet.stake,
            odds: bet.odds,
            occured_at: Utc::now(),
        };

        match serde_json::to_string(&event) {
            Ok(payload) => {
                let delivery = self
                    .state
                    .producer
                    .send(
                        FutureRecord::to("bets.placed")
                            .key(&event.event_id.to_string())
                            .payload(&payload),
                        std::time::Duration::from_secs(5),
                    )
                    .await;

                match delivery {
                    Ok((partition, offset)) => {
                        tracing::info!(
                            partition,
                            offset,
                            event_id = %event.event_id,
                            "BetPlaced published"
                        )
                    }
                    Err((e, _)) => tracing::error!(
                        error = ?e, event_id = %event.event_id,
                        "Failed to publish BetPlaced event"
                    ),
                }
            }

            Err(e) => {
                tracing::error!(error = ?e, "Failed to serialize BetPlaced");
            }
        }
        Ok(PlaceBetResponse::Status201_BetPlaced(Self::to_model_bet(
            bet,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn to_model_bet_maps_all_fields() {
        let id = Uuid::new_v4();
        let event_id = Uuid::new_v4();
        let created_at = Utc::now();

        let bet = Bet {
            id,
            event_id,
            stake: 25.0,
            odds: 2.0,
            created_at,
        };

        let model = ApiImpl::to_model_bet(bet);
        assert_eq!(model.id, id);
        assert_eq!(model.event_id, event_id);
        assert_eq!(model.stake, 25.0);
        assert_eq!(model.odds, 2.0);
        assert_eq!(model.created_at, created_at);
    }

    #[test]
    fn err_body_wraps_message() {
        let body = ApiImpl::err_body("bet not found");
        assert_eq!(body.error, "bet not found");
    }

    #[test]
    fn err_body_accepts_owned_strings() {
        let msg = format!("bet {} not found", Uuid::new_v4());
        let body = ApiImpl::err_body(msg.clone());
        assert_eq!(body.error, msg);
    }
}
