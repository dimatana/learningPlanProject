use std::sync::Arc;

use async_trait::async_trait;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use chrono::Utc;
use headers::Host;
use http::Method;
use rdkafka::producer::FutureRecord;

use bet_api_generated::apis;
use bet_api_generated::apis::default::{
    Default as DefaultApi, GetBetResponse, GetHealthResponse, ListBetsResponse, PlaceBetResponse,
};
use bet_api_generated::models;

use crate::domain::{self, Bet};
use crate::error::AppError;
use crate::repository;
use crate::state::AppState;


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
        let bets = repository::list_bets(&self.state.pool)
            .await
            .map_err(|_| AppError::DatabaseError)?;

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

        let bet = repository::fetch_bet_by_id(&self.state.pool, id)
            .await
            .map_err(|_| AppError::DatabaseError)?;

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
        let valid = match domain::validate_place_bet(body.clone()) {
            Ok(v) => v,
            Err(AppError::InvalidStake(v)) => {
                return Ok(PlaceBetResponse::Status422_InvalidRequest(Self::err_body(
                    format!("Invalid stake: {v}"),
                )));
            }
            Err(AppError::InvalidOdds(v)) => {
                return Ok(PlaceBetResponse::Status422_InvalidRequest(Self::err_body(
                    format!("Invalid odds: {v}"),
                )));
            }
            Err(_) => {
                return Ok(PlaceBetResponse::Status500_InternalServerError(
                    Self::err_body("Internal server error"),
                ));
            }
        };

        let bet =
            repository::insert_bet(&self.state.pool, valid.event_id, valid.stake, valid.odds)
                .await
                .map_err(|_| AppError::DatabaseError)?;

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
                        FutureRecord::to("bets-events")
                            .key(&bet.id.to_string())
                            .payload(&payload),
                        std::time::Duration::from_secs(5),
                    )
                    .await;
                if let Err((e, _)) = delivery {
                    tracing::error!("failed to publish event: {:?}", e);
                }
            }
            Err(e) => {
                tracing::error!("failed to serialize bet event: {:?}", e);
            }
        }

        Ok(PlaceBetResponse::Status201_BetPlaced(Self::to_model_bet(bet)))
    }
}