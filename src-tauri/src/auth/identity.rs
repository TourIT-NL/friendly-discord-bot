// src-tauri/src/auth/identity.rs

use super::types::DiscordUser;
use crate::api::rate_limiter::ApiHandle;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::vault::{DiscordIdentity, Vault};
use tauri::{AppHandle, Emitter, Manager, Window};

#[tauri::command]
pub async fn login_with_user_token(
    app_handle: AppHandle,
    window: Window,
    token: String,
) -> Result<DiscordUser, AppError> {
    login_with_token_internal(app_handle, window, token, false).await
}

pub async fn login_with_token_internal(
    app_handle: AppHandle,
    window: Window,
    token: String,
    is_oauth: bool,
) -> Result<DiscordUser, AppError> {
    let token = token
        .trim()
        .trim_start_matches("Bearer ")
        .trim_matches('"')
        .to_string();
    let user_profile = validate_token(&app_handle, &token, is_oauth).await?;

    Vault::save_identity(
        &app_handle,
        DiscordIdentity {
            id: user_profile.id.clone(),
            username: user_profile.username.clone(),
            token: token.clone(),
            is_oauth,
        },
    )?;

    let _ = window.emit("auth_success", user_profile.clone());
    Ok(user_profile)
}

pub async fn validate_token(
    app_handle: &AppHandle,
    token: &str,
    is_bearer: bool,
) -> Result<DiscordUser, AppError> {
    let api_handle = app_handle.state::<ApiHandle>();
    let response_value = api_handle
        .send_request(
            reqwest::Method::GET,
            "https://discord.com/api/v10/users/@me",
            None,
            token,
            is_bearer,
        )
        .await?; // Will return serde_json::Value if successful

    serde_json::from_value(response_value).map_err(AppError::from)
}

#[tauri::command]
pub async fn save_discord_credentials(
    app_handle: AppHandle,
    client_id: String,
    client_secret: String,
) -> Result<(), AppError> {
    Vault::set_credential(&app_handle, "client_id", client_id.trim())?;
    Vault::set_credential(&app_handle, "client_secret", client_secret.trim())?;
    Logger::info(&app_handle, "[Vault] Discord credentials updated", None);
    Ok(())
}

#[tauri::command]
pub async fn list_identities(app_handle: AppHandle) -> Result<Vec<DiscordIdentity>, AppError> {
    Ok(Vault::list_identities(&app_handle))
}

#[tauri::command]
pub async fn switch_identity(
    app_handle: AppHandle,
    window: Window,
    id: String,
) -> Result<DiscordUser, AppError> {
    let identities = Vault::list_identities(&app_handle);
    let identity = identities
        .iter()
        .find(|i| i.id == id)
        .ok_or_else(|| AppError {
            user_message: "Identity not found.".into(),
            ..Default::default()
        })?;
    Logger::info(
        &app_handle,
        &format!("[Auth] Switching to identity: {}", identity.username),
        None,
    );
    login_with_token_internal(
        app_handle,
        window,
        identity.token.clone(),
        identity.is_oauth,
    )
    .await
}

#[tauri::command]
pub async fn remove_identity(app_handle: AppHandle, id: String) -> Result<(), AppError> {
    Vault::remove_identity(&app_handle, &id)
}

#[tauri::command]
pub async fn get_current_user(
    app_handle: AppHandle,
    window: Window,
) -> Result<DiscordUser, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let user_profile = validate_token(&app_handle, &token, is_bearer).await?;
    let _ = window.emit("auth_success", user_profile.clone());
    Ok(user_profile)
}
