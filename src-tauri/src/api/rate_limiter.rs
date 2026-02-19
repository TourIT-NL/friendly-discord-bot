// src-tauri/src/api/rate_limiter.rs

use tokio::sync::{mpsc, oneshot, Mutex};
use std::sync::Arc;
use std::time::Duration;
use reqwest::{Client, Method, Response, header};
use tracing::{info, warn, error};
use crate::core::error::AppError;

/// Represents a pending API request
pub struct ApiRequest {
    pub method: Method,
    pub url: String,
    pub body: Option<serde_json::Value>,
    pub auth_token: String,
    pub is_bearer: bool, // true for OAuth2 (Bearer), false for User Tokens (Direct)
    pub response_tx: oneshot::Sender<Result<reqwest::Response, AppError>>,
}

/// Information about the current rate limit window
#[derive(Clone, Default, Debug)]
pub struct RateLimitInfo {
    pub remaining: u32,
    pub reset_after: Duration,
    pub limit: u32,
}

/// The actor's state
pub struct RateLimiterActor {
    /// A queue for incoming requests
    inbox: mpsc::Receiver<ApiRequest>,
    /// The HTTP client
    client: Client,
    /// Information about the current rate limit window
    rate_limit_info: Arc<Mutex<RateLimitInfo>>,
}

impl RateLimiterActor {
    pub fn new(inbox: mpsc::Receiver<ApiRequest>) -> Self {
        Self {
            inbox,
            client: Client::new(),
            rate_limit_info: Arc::new(Mutex::new(RateLimitInfo {
                remaining: 1,
                reset_after: Duration::from_secs(0),
                limit: 1,
            })),
        }
    }

    pub async fn run(&mut self) {
        info!("RateLimiterActor starting...");
        while let Some(request) = self.inbox.recv().await {
            let mut info = self.rate_limit_info.lock().await;
            
            if info.remaining == 0 {
                let wait_time = info.reset_after;
                if wait_time > Duration::from_secs(0) {
                    warn!("Rate limit reached. Waiting for {:?} before next request...", wait_time);
                    tokio::time::sleep(wait_time).await;
                    info.remaining = 1; 
                }
            }
            drop(info);

            // Execute the request with correct auth header
            let mut req_builder = self.client.request(request.method.clone(), &request.url);
            
            if request.is_bearer {
                req_builder = req_builder.bearer_auth(&request.auth_token);
            } else {
                req_builder = req_builder.header(header::AUTHORIZATION, &request.auth_token);
            }

            if let Some(body) = request.body {
                req_builder = req_builder.json(&body);
            }

            match req_builder.send().await {
                Ok(response) => {
                    self.update_rate_limit_info(&response).await;
                    let _ = request.response_tx.send(Ok(response));
                }
                Err(e) => {
                    error!("Request error for {}: {}", request.url, e);
                    let _ = request.response_tx.send(Err(AppError::from(e)));
                }
            }
        }
        info!("RateLimiterActor shutting down.");
    }

    async fn update_rate_limit_info(&self, response: &Response) {
        let headers = response.headers();
        let mut info = self.rate_limit_info.lock().await;

        if let Some(remaining) = headers.get("X-RateLimit-Remaining")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok()) {
            info.remaining = remaining;
        }

        if let Some(reset_after) = headers.get("X-RateLimit-Reset-After")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<f32>().ok()) {
            info.reset_after = Duration::from_secs_f32(reset_after);
        }

        if let Some(limit) = headers.get("X-RateLimit-Limit")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok()) {
            info.limit = limit;
        }
    }
}

/// A handle to send requests to the RateLimiterActor
#[derive(Clone)]
pub struct ApiHandle {
    tx: mpsc::Sender<ApiRequest>,
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
    ) -> Result<reqwest::Response, AppError> {
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
            user_message: "Internal error: Failed to communicate with rate limiter.".to_string(),
            error_code: "rate_limiter_channel_failure".to_string(),
            technical_details: None,
        })?;

        response_rx.await?
    }
}
