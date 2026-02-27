// src-tauri/src/api/discord/billing.rs

use crate::api::rate_limiter::ApiHandle;
use crate::core::error::AppError;
use crate::core::vault::Vault;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn fetch_payment_sources(app_handle: AppHandle) -> Result<serde_json::Value, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();

    api_handle
        .send_request_json(
            reqwest::Method::GET,
            "https://discord.com/api/v9/users/@me/billing/payment-sources",
            None,
            &token,
            is_bearer,
            None,
        )
        .await
}

#[tauri::command]
pub async fn fetch_billing_subscriptions(
    app_handle: AppHandle,
) -> Result<serde_json::Value, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();

    api_handle
        .send_request_json(
            reqwest::Method::GET,
            "https://discord.com/api/v9/users/@me/billing/subscriptions",
            None,
            &token,
            is_bearer,
            None,
        )
        .await
}

#[tauri::command]
pub async fn fetch_entitlements(app_handle: AppHandle) -> Result<serde_json::Value, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();

    api_handle
        .send_request_json(
            reqwest::Method::GET,
            "https://discord.com/api/v9/users/@me/entitlements?can_multiline=true",
            None,
            &token,
            is_bearer,
            None,
        )
        .await
}
