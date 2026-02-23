// src-tauri/src/api/discord/security.rs

use crate::api::rate_limiter::ApiHandle;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::vault::Vault;
use reqwest::Method;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn fetch_oauth_tokens(app_handle: AppHandle) -> Result<serde_json::Value, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();

    Logger::info(
        &app_handle,
        "[SECURITY] Auditing third-party OAuth access",
        None,
    );

    api_handle
        .send_request(
            Method::GET,
            "https://discord.com/api/v9/oauth2/tokens",
            None,
            &token,
            is_bearer,
        )
        .await
}

#[tauri::command]
pub async fn revoke_oauth_token(app_handle: AppHandle, token_id: String) -> Result<(), AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();

    Logger::warn(
        &app_handle,
        &format!("[SECURITY] Revoking authorized app: {}", token_id),
        None,
    );

    let _ = api_handle
        .send_request(
            Method::DELETE,
            &format!("https://discord.com/api/v9/oauth2/tokens/{}", token_id),
            None,
            &token,
            is_bearer,
        )
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn fetch_application_identities(
    app_handle: AppHandle,
) -> Result<serde_json::Value, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();

    Logger::info(
        &app_handle,
        "[SECURITY] Fetching authorized application identities",
        None,
    );

    api_handle
        .send_request(
            Method::GET,
            "https://discord.com/api/v9/users/@me/application-identities",
            None,
            &token,
            is_bearer,
        )
        .await
}
