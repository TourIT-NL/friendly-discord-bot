// src-tauri/src/api/discord/privacy.rs

use crate::api::rate_limiter::ApiHandle;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::op_manager::OperationManager;
use crate::core::vault::Vault;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn stealth_privacy_wipe(app_handle: AppHandle) -> Result<(), AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    if is_bearer {
        return Err(AppError {
            user_message: "Stealth Mode restricted in Official Gate.".into(),
            ..Default::default()
        });
    }

    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.prepare();
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    Logger::info(
        &app_handle,
        "[STEALTH] Privacy protocol execution loop active...",
        None,
    );

    // 1. Wipe Custom Status
    op_manager.state.wait_if_paused().await;
    if !op_manager.state.should_abort.load(Ordering::SeqCst) {
        Logger::debug(&app_handle, "[STEALTH] Nullifying custom status", None);
        let _ = api_handle
            .send_request(
                reqwest::Method::PATCH,
                "https://discord.com/api/v9/users/@me/settings",
                Some(serde_json::json!({ "custom_status": null })),
                &token,
                is_bearer,
            )
            .await;
    }

    // 2. Global DM Disable
    op_manager.state.wait_if_paused().await;
    if !op_manager.state.should_abort.load(Ordering::SeqCst) {
        Logger::debug(&app_handle, "[STEALTH] Updating DM buffer protocols", None);
        let _ = api_handle
            .send_request(
                reqwest::Method::PATCH,
                "https://discord.com/api/v9/users/@me/settings",
                Some(serde_json::json!({ "default_guilds_restricted": true })),
                &token,
                is_bearer,
            )
            .await;
    }

    // 3. Presence Privacy
    op_manager.state.wait_if_paused().await;
    if !op_manager.state.should_abort.load(Ordering::SeqCst) {
        Logger::debug(
            &app_handle,
            "[STEALTH] Masking presence game/activity data",
            None,
        );
        let _ = api_handle
            .send_request(
                reqwest::Method::PATCH,
                "https://discord.com/api/v9/users/@me/settings",
                Some(serde_json::json!({ "show_current_game": false, "restricted_guilds": [] })),
                &token,
                is_bearer,
            )
            .await;
    }

    op_manager.state.reset();
    Logger::info(
        &app_handle,
        "[STEALTH] Privacy protocol sequence complete.",
        None,
    );
    Ok(())
}

#[tauri::command]
pub async fn nitro_stealth_wipe(app_handle: AppHandle) -> Result<(), AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    if is_bearer {
        return Err(AppError {
            user_message: "Nitro Stealth restricted in Official Gate.".into(),
            ..Default::default()
        });
    }

    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.prepare();
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    Logger::info(
        &app_handle,
        "[NITRO] Initiating stealth wipe protocol for premium metadata",
        None,
    );

    // 1. Clear About Me
    op_manager.state.wait_if_paused().await;
    if !op_manager.state.should_abort.load(Ordering::SeqCst) {
        Logger::debug(&app_handle, "[NITRO] Nullifying bio/about-me", None);
        let _ = api_handle
            .send_request(
                reqwest::Method::PATCH,
                "https://discord.com/api/v9/users/@me",
                Some(serde_json::json!({ "bio": "" })),
                &token,
                is_bearer,
            )
            .await;
    }

    // 2. Clear Pronouns
    op_manager.state.wait_if_paused().await;
    if !op_manager.state.should_abort.load(Ordering::SeqCst) {
        Logger::debug(&app_handle, "[NITRO] Nullifying profile pronouns", None);
        let _ = api_handle
            .send_request(
                reqwest::Method::PATCH,
                "https://discord.com/api/v9/users/@me/settings",
                Some(serde_json::json!({ "pronouns": "" })),
                &token,
                is_bearer,
            )
            .await;
    }

    // 3. Reset Banner
    op_manager.state.wait_if_paused().await;
    if !op_manager.state.should_abort.load(Ordering::SeqCst) {
        Logger::debug(&app_handle, "[NITRO] Nullifying profile banner", None);
        let _ = api_handle
            .send_request(
                reqwest::Method::PATCH,
                "https://discord.com/api/v9/users/@me",
                Some(serde_json::json!({ "banner": null })),
                &token,
                is_bearer,
            )
            .await;
    }

    op_manager.state.reset();
    Logger::info(&app_handle, "[NITRO] Stealth wipe sequence complete.", None);
    Ok(())
}

#[tauri::command]
pub async fn trigger_data_harvest(app_handle: AppHandle) -> Result<serde_json::Value, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();

    Logger::info(&app_handle, "[GDPR] Initiating data harvest request", None);

    api_handle
        .send_request(
            reqwest::Method::POST,
            "https://discord.com/api/v9/users/@me/harvest",
            Some(serde_json::json!({
                "backends": ["Account", "Analytics", "Activities", "Ads", "Messages", "Servers", "Zendesk"]
            })),
            &token,
            is_bearer,
        )
        .await
}

#[tauri::command]
pub async fn get_harvest_status(app_handle: AppHandle) -> Result<serde_json::Value, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();

    api_handle
        .send_request(
            reqwest::Method::GET,
            "https://discord.com/api/v9/users/@me/harvest",
            None,
            &token,
            is_bearer,
        )
        .await
}

#[tauri::command]
pub async fn set_max_privacy_settings(app_handle: AppHandle) -> Result<(), AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();

    Logger::info(&app_handle, "[GDPR] Applying maximum privacy hardening via Protobuf", None);

    let proto_bytes = crate::core::protobuf::encode_max_privacy();
    let proto_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, proto_bytes);

    let _ = api_handle
        .send_request(
            reqwest::Method::PATCH,
            "https://discord.com/api/v9/users/@me/settings-proto/1",
            Some(serde_json::json!({ "settings": proto_b64 })),
            &token,
            is_bearer,
        )
        .await?;

    Ok(())
}
