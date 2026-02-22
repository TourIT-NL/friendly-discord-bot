// src-tauri/src/api/rate_limiter/handle.rs

use crate::api::rate_limiter::types::ApiRequest;
use crate::core::error::AppError;
use reqwest::Method;
use tokio::sync::{mpsc, oneshot};

#[derive(Clone)]
pub struct ApiHandle {
    pub(crate) tx: mpsc::Sender<ApiRequest>,
}

impl ApiHandle {
    pub fn new(tx: mpsc::Sender<ApiRequest>) -> Self {
        Self { tx }
    }

    pub async fn send_request(
        &self,
        method: Method,
        url: &str,
        body: Option<serde_json::Value>,
        auth_token: &str,
        is_bearer: bool,
    ) -> Result<serde_json::Value, AppError> {
        let (response_tx, response_rx) = oneshot::channel();

        let api_request = ApiRequest {
            method,
            url: url.to_string(),
            body,
            auth_token: auth_token.to_string(),
            is_bearer,
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
}
