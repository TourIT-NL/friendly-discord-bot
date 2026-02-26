// src-tauri/src/api/discord/billing.rs

use crate::api::rate_limiter::{ApiHandle, types::ApiResponseContent};
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::vault::Vault;
use reqwest::Method;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn fetch_payment_sources(app_handle: AppHandle) -> Result<serde_json::Value, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();

    Logger::info(
        &app_handle,
        "[ACCOUNT] Fetching stored payment methods",
        None,
    );

    let response_content = api_handle
        .send_request(
            Method::GET,
            "https://discord.com/api/v9/users/@me/billing/payment-sources",
            None,
            &token,
            is_bearer,
            false,
        )
        .await?;

    match response_content {
        ApiResponseContent::Json(json) => Ok(json),
        ApiResponseContent::Bytes(_) => Err(AppError::new(
            "Expected JSON, received raw bytes",
            "unexpected_response_type",
        )),
    }
}

#[tauri::command]
pub async fn fetch_billing_subscriptions(
    app_handle: AppHandle,
) -> Result<serde_json::Value, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();

    Logger::info(&app_handle, "[ACCOUNT] Auditing active subscriptions", None);

    let response_content = api_handle
        .send_request(
            Method::GET,
            "https://discord.com/api/v9/users/@me/billing/subscriptions",
            None,
            &token,
            is_bearer,
            false,
        )
        .await?;

    match response_content {
        ApiResponseContent::Json(json) => Ok(json),
        ApiResponseContent::Bytes(_) => Err(AppError::new(
            "Expected JSON, received raw bytes",
            "unexpected_response_type",
        )),
    }
}

#[tauri::command]
pub async fn fetch_entitlements(app_handle: AppHandle) -> Result<serde_json::Value, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();

    Logger::info(&app_handle, "[ACCOUNT] Fetching entitlements", None);

    let response_content = api_handle
        .send_request(
            Method::GET,
            "https://discord.com/api/v9/users/@me/entitlements",
            None,
            &token,
            is_bearer,
            false,
        )
        .await?;

    match response_content {
        ApiResponseContent::Json(json) => Ok(json),
        ApiResponseContent::Bytes(_) => Err(AppError::new(
            "Expected JSON, received raw bytes",
            "unexpected_response_type",
        )),
    }
}
