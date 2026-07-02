use std::sync::Arc;

use async_trait::async_trait;
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use headers::Host;
use http::Method;
use trading_api_generated::apis;
use trading_api_generated::apis::default::{
    Default as DefaultApi, GetEventResponse, GetHealthResponse,
};
use trading_api_generated::models;

use crate::domain::Event;
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

    fn to_model_event(e: Event) -> models::Event {
        models::Event {
            event_id: e.event_id,
            name: e.name,
            bets_placed: e
                .bets_placed
                .try_into()
                .expect("bets_placed should be non-negative"),
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

    async fn get_event(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        path_params: &models::GetEventPathParams,
    ) -> Result<GetEventResponse, AppError> {
        let id = path_params.id;
        let event = repository::fetch_event_by_id(&self.state.pool, id)
            .await
            .map_err(|_| AppError::DatabaseError)?;

        match event {
            Some(e) => Ok(GetEventResponse::Status200_EventFound(
                Self::to_model_event(e),
            )),
            None => Ok(GetEventResponse::Status404_EventNotFound(Self::err_body(
                format!("Event {id} not found"),
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn to_model_event_maps_all_fields() {
        let event_id = Uuid::new_v4();
        let event = Event {
            event_id,
            name: "Real Madrid vs Barcelona".to_string(),
            bets_placed: 12,
        };

        let model = ApiImpl::to_model_event(event);

        assert_eq!(model.event_id, event_id);
        assert_eq!(model.name, "Real Madrid vs Barcelona");
        assert_eq!(model.bets_placed, 12);
    }

    #[test]
    fn to_model_event_maps_zero_bets_placed() {
        let event = Event {
            event_id: Uuid::new_v4(),
            name: "New event".to_string(),
            bets_placed: 0,
        };

        let model = ApiImpl::to_model_event(event);
        assert_eq!(model.bets_placed, 0);
    }

    #[test]
    #[should_panic(expected = "bets_placed should be non-negative")]
    fn to_model_event_panics_on_negative_bets_placed() {
        let event = Event {
            event_id: Uuid::new_v4(),
            name: "Corrupt row".to_string(),
            bets_placed: -1,
        };

        ApiImpl::to_model_event(event);
    }

    #[test]
    fn err_body_wraps_message() {
        let body = ApiImpl::err_body("Event not found");
        assert_eq!(body.error, "Event not found");
    }
}
