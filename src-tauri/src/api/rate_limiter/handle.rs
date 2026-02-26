// src-tauri/src/api/rate_limiter/handle.rs

use crate::api::rate_limiter::types::{ApiRequest, ApiResponseContent};
use crate::core::error::AppError;
use reqwest::Method;
use tokio::sync::{mpsc, oneshot};

/// A thread-safe handle to the central RateLimiterActor.
/// Used by other modules to dispatch API requests without managing rate limits themselves.
#[derive(Clone)]
pub struct ApiHandle {
    pub(crate) tx: mpsc::Sender<ApiRequest>,
}

impl ApiHandle {
    pub fn new(tx: mpsc::Sender<ApiRequest>) -> Self {
        Self { tx }
    }

    /// Dispatches a raw HTTP request through the rate limiter.
    /// Supports both JSON and binary (raw bytes) response types.
    pub async fn send_request(
        &self,
        method: Method,
        url: &str,
        body: Option<serde_json::Value>,
        auth_token: &str,
        is_bearer: bool,
        return_raw_bytes: bool,
    ) -> Result<ApiResponseContent, AppError> {
        let (response_tx, response_rx) = oneshot::channel();

        let api_request = ApiRequest::Standard {
            method,
            url: url.to_string(),
            body,
            auth_token: auth_token.to_string(),
            is_bearer,
            return_raw_bytes,
            response_tx,
        };

        self.tx.send(api_request).await.map_err(|_| AppError {
            user_message: "Rate limiter connection failure.".to_string(),
            error_code: "limiter_offline".to_string(),
            technical_details: None,
        })?;

        response_rx.await.map_err(|_| AppError {
            user_message: "Rate limiter communication timeout.".to_string(),
            error_code: "limiter_timeout".to_string(),
            technical_details: None,
        })?
    }

    /// High-level helper to send a JSON request and automatically parse the response.
    pub async fn send_request_json(
        &self,
        method: Method,
        url: &str,
        body: Option<serde_json::Value>,
        auth_token: &str,
        is_bearer: bool,
    ) -> Result<serde_json::Value, AppError> {
        self.send_request(method, url, body, auth_token, is_bearer, false)
            .await?
            .json()
    }

    /// Signals the RateLimiterActor to rebuild its HTTP client.
    /// Used when proxy settings or fingerprints are updated.
    pub async fn rebuild_client(&self) -> Result<(), AppError> {
        self.tx
            .send(ApiRequest::RebuildClient)
            .await
            .map_err(|_| AppError {
                user_message: "Rate limiter connection failure.".to_string(),
                error_code: "limiter_offline".to_string(),
                ..Default::default()
            })
    }
}
