// src-tauri/src/core/forensics/honey.rs

use crate::api::rate_limiter::types::{ApiRequest, StandardRequest};
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::vault::Vault;
use rand::Rng;
use reqwest::Method;
use tauri::{AppHandle, Manager};
use tokio::sync::oneshot;

/// Honey-traffic noise generator for telemetry evasion.
/// Shrouds destructive actions in a cloud of benign user behavior.
pub struct HoneyManager;

impl HoneyManager {
    /// Generates a random non-destructive Discord API request.
    /// Targeted endpoints are common browser-based telemetry or metadata fetches.
    pub fn generate_noise_request(app: &AppHandle) -> Result<ApiRequest, AppError> {
        let (token, is_bearer) = Vault::get_active_token(app)?;
        let mut rng = rand::thread_rng();

        let endpoints = [
            (Method::GET, "https://discord.com/api/v9/users/@me/library"),
            (
                Method::GET,
                "https://discord.com/api/v9/users/@me/applications",
            ),
            (
                Method::GET,
                "https://discord.com/api/v9/users/@me/guild-events",
            ),
            (
                Method::GET,
                "https://discord.com/api/v9/users/@me/connections",
            ),
            (Method::GET, "https://discord.com/api/v9/experiments"),
            (
                Method::GET,
                "https://discord.com/api/v9/users/@me/billing/country-code",
            ),
        ];

        let (method, url) = endpoints[rng.gen_range(0..endpoints.len())].clone();
        let (tx, _) = oneshot::channel();

        Ok(ApiRequest::Standard(Box::new(StandardRequest {
            method,
            url: url.to_string(),
            body: None,
            auth_token: token,
            is_bearer,
            return_raw_bytes: false,
            response_tx: tx,
            referer: Some("https://discord.com/channels/@me".to_string()),
            locale: None,
            timezone: None,
            profile: None,
        })))
    }

    /// Injects noise into the rate limiter queue if an operation is active.
    pub async fn pulse_noise(app: &AppHandle) {
        let api_handle = match app.try_state::<crate::api::rate_limiter::ApiHandle>() {
            Some(h) => h,
            None => return,
        };

        if let Ok(noise) = Self::generate_noise_request(app) {
            Logger::trace(
                app,
                "[HONEY] Injecting telemetry noise to shroud activity",
                None,
            );
            let _ = api_handle.tx.send(noise).await;
        }
    }
}
