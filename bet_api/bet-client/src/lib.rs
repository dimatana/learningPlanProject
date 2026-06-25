use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct Client {
    base_url: String,
    http: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceBetRequest {
    pub event_id: Uuid,
    pub stake: f64,
    pub odds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bet {
    pub id: Uuid,
    pub event_id: Uuid,
    pub stake: f64,
    pub odds: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("api returned {status}: {body}")]
    Api { status: StatusCode, body: String },
}

impl Client {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            http: reqwest::Client::new(),
        }
    }
    pub async fn get_health(&self) -> Result<String, Error> {
        let response = self
            .http
            .get(&format!("{}/health", self.base_url))
            .send()
            .await?;
        Self::expect_success(response)
            .await?
            .text()
            .await
            .map_err(Error::from)
    }
    pub async fn place_bet(&self, request: &PlaceBetRequest) -> Result<Bet, Error> {
        let response = self
            .http
            .post(format!("{}/bets", self.base_url))
            .json(request)
            .send()
            .await?;

        Self::expect_success(response)
            .await?
            .json()
            .await
            .map_err(Error::from)
    }
    pub async fn get_bet(&self, id: Uuid) -> Result<Bet, Error> {
        let response = self
            .http
            .get(format!("{}/bets/{}", self.base_url, id))
            .send()
            .await?;

        Self::expect_success(response)
            .await?
            .json()
            .await
            .map_err(Error::from)
    }
    pub async fn list_bets(&self) -> Result<Vec<Bet>, Error> {
        let response = self
            .http
            .get(format!("{}/bets", self.base_url))
            .send()
            .await?;

        Self::expect_success(response)
            .await?
            .json()
            .await
            .map_err(Error::from)
    }
    async fn expect_success(response: reqwest::Response) -> Result<reqwest::Response, Error> {
        let status = response.status();

        if status.is_success() {
            return Ok(response);
        }

        let body = match response.json::<ErrorResponse>().await {
            Ok(error) => error.error,
            Err(error) => error.to_string(),
        };

        Err(Error::Api { status, body })
    }
}
