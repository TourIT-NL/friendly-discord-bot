// src-tauri/src/api/rate_limiter/handle.rs

use crate::api::rate_limiter::fingerprint::BrowserProfile;
use crate::api::rate_limiter::types::{ApiRequest, ApiResponseContent, StandardRequest};
use crate::core::error::AppError;
use reqwest::Method;
use tokio::sync::{mpsc, oneshot};

#[derive(Default)]
pub struct RequestConfig {
    pub referer: Option<String>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
    pub profile: Option<BrowserProfile>,
    pub return_raw_bytes: bool,
}

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
        config: RequestConfig,
    ) -> Result<ApiResponseContent, AppError> {
        let (response_tx, response_rx) = oneshot::channel();

        self.tx
            .send(ApiRequest::Standard(Box::new(StandardRequest {
                method,
                url: url.to_string(),
                body,
                auth_token: auth_token.to_string(),
                is_bearer,
                return_raw_bytes: config.return_raw_bytes,
                response_tx,
                referer: config.referer,
                locale: config.locale,
                timezone: config.timezone,
                profile: config.profile,
            })))
            .await
            .map_err(|_| AppError::new("Limiter offline", "limiter_offline"))?;

        response_rx
            .await
            .map_err(|_| AppError::new("Limiter timeout", "limiter_timeout"))?
    }

    pub async fn send_request_json(
        &self,
        method: Method,
        url: &str,
        body: Option<serde_json::Value>,
        auth_token: &str,
        is_bearer: bool,
        referer: Option<String>,
    ) -> Result<serde_json::Value, AppError> {
        self.send_request(
            method,
            url,
            body,
            auth_token,
            is_bearer,
            RequestConfig {
                referer,
                ..Default::default()
            },
        )
        .await?
        .json()
    }

    pub async fn rebuild_client(&self) -> Result<(), AppError> {
        self.tx
            .send(ApiRequest::RebuildClient)
            .await
            .map_err(|_| AppError::new("Limiter offline", "limiter_offline"))
    }
}
