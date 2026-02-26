// src-tauri/src/api/rate_limiter/types.rs

use crate::core::error::AppError;
use bytes::Bytes;
use reqwest::Method;
use serde_json;
use std::time::Instant;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum ApiResponseContent {
    Json(serde_json::Value),
    Bytes(Bytes),
}

impl ApiResponseContent {
    pub fn json(self) -> Result<serde_json::Value, AppError> {
        match self {
            ApiResponseContent::Json(v) => Ok(v),
            ApiResponseContent::Bytes(_) => Err(AppError {
                user_message: "Expected JSON, got bytes".to_string(),
                error_code: "api_type_mismatch".to_string(),
                ..Default::default()
            }),
        }
    }
}

/// Represents a pending API request or control signal
pub enum ApiRequest {
    Standard {
        method: Method,
        url: String,
        body: Option<serde_json::Value>,
        auth_token: String,
        is_bearer: bool,
        return_raw_bytes: bool,
        response_tx: oneshot::Sender<Result<ApiResponseContent, AppError>>,
    },
    RebuildClient,
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
