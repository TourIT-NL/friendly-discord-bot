// src-tauri/src/api/rate_limiter/types.rs

use crate::api::rate_limiter::fingerprint::BrowserProfile;
use crate::core::error::AppError;
use bytes::Bytes;
use reqwest::Method;
use serde_json;
use std::time::Instant;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum ApiResponseContent {
    Json(serde_json::Value),
    #[allow(dead_code)]
    Bytes(Bytes),
}

impl ApiResponseContent {
    pub fn json(self) -> Result<serde_json::Value, AppError> {
        match self {
            ApiResponseContent::Json(v) => Ok(v),
            _ => Err(AppError::new("Expected JSON response", "api_type_mismatch")),
        }
    }
}

pub struct StandardRequest {
    pub method: Method,
    pub url: String,
    pub body: Option<serde_json::Value>,
    pub auth_token: String,
    pub is_bearer: bool,
    pub return_raw_bytes: bool,
    pub response_tx: oneshot::Sender<Result<ApiResponseContent, AppError>>,
    pub referer: Option<String>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
    pub profile: Option<BrowserProfile>,
}

pub enum ApiRequest {
    Standard(Box<StandardRequest>),
    RebuildClient,
}

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
