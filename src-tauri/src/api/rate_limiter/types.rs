// src-tauri/src/api/rate_limiter/types.rs

use crate::core::error::AppError;
use reqwest::Method;
use std::time::Instant;
use tokio::sync::oneshot;

/// Represents a pending API request
pub struct ApiRequest {
    pub method: Method,
    pub url: String,
    pub body: Option<serde_json::Value>,
    pub auth_token: String,
    pub is_bearer: bool,
    pub response_tx: oneshot::Sender<Result<serde_json::Value, AppError>>,
}

/// Information about a rate limit bucket
#[derive(Clone, Debug)]
pub struct BucketInfo {
    pub remaining: u32,
    pub reset_at: Instant,
    pub limit: u32,
    pub consecutive_429s: u32,
}

impl Default for BucketInfo {
    fn default() -> Self {
        Self {
            remaining: 1,
            reset_at: Instant::now(),
            limit: 1,
            consecutive_429s: 0,
        }
    }
}
