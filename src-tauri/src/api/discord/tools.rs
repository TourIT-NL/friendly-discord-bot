// src-tauri/src/api/discord/tools.rs

use crate::api::rate_limiter::ApiHandle;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::op_manager::OperationManager;
use crate::core::vault::Vault;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, Window};
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
pub async fn bury_audit_log(
    app_handle: AppHandle,
    window: Window,
    guild_id: String,
    channel_id: String,
) -> Result<(), AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.prepare();
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    if is_bearer {
        return Err(AppError::new(
            "Audit Log Burial restricted",
            "auth_mismatch",
        ));
    }

    Logger::info(
        &app_handle,
        &format!("[AUDIT] Starting burial sequence in guild {}", guild_id),
        None,
    );

    let original_channel_json = api_handle
        .send_request_json(
            reqwest::Method::GET,
            &format!("https://discord.com/api/v9/channels/{}", channel_id),
            None,
            &token,
            is_bearer,
            None,
        )
        .await?;

    let original_channel_name = original_channel_json["name"]
        .as_str()
        .unwrap_or("general")
        .to_string();

    for i in 0..10 {
        op_manager.state.wait_if_paused().await;
        if op_manager.state.should_abort.load(Ordering::SeqCst) {
            break;
        }

        let new_name = format!("{}-temp-{}", original_channel_name, i);
        let _ = api_handle
            .send_request_json(
                reqwest::Method::PATCH,
                &format!("https://discord.com/api/v9/channels/{}", channel_id),
                Some(serde_json::json!({ "name": new_name })),
                &token,
                is_bearer,
                None,
            )
            .await?;

        let _ = window.emit("audit_log_progress", serde_json::json!({ "current": i + 1, "total": 20, "status": format!("Burying phase {}", i) }));
        tokio::time::sleep(Duration::from_millis(500)).await;

        let _ = api_handle
            .send_request_json(
                reqwest::Method::PATCH,
                &format!("https://discord.com/api/v9/channels/{}", channel_id),
                Some(serde_json::json!({ "name": &original_channel_name })),
                &token,
                is_bearer,
                None,
            )
            .await?;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    op_manager.state.reset();
    let _ = window.emit("audit_log_complete", ());
    Ok(())
}

#[tauri::command]
pub async fn webhook_ghosting(
    app_handle: AppHandle,
    window: Window,
    guild_id: String,
) -> Result<(), AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.prepare();
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    if is_bearer {
        return Err(AppError::new(
            "Webhook Ghosting restricted",
            "auth_mismatch",
        ));
    }

    Logger::info(
        &app_handle,
        &format!("[WEBHOOK] Auditing hooks in node {}", guild_id),
        None,
    );

    let webhooks_json = api_handle
        .send_request_json(
            reqwest::Method::GET,
            &format!("https://discord.com/api/v9/guilds/{}/webhooks", guild_id),
            None,
            &token,
            is_bearer,
            None,
        )
        .await?;

    let webhooks: Vec<serde_json::Value> =
        serde_json::from_value(webhooks_json).map_err(AppError::from)?;

    let mut deleted_webhooks = 0;
    for webhook in &webhooks {
        op_manager.state.wait_if_paused().await;
        if op_manager.state.should_abort.load(Ordering::SeqCst) {
            break;
        }

        let webhook_id = webhook["id"].as_str().unwrap_or_default();
        let webhook_creator_id = webhook["user"]["id"].as_str().unwrap_or_default();
        let user_id_from_token = token.split('.').next().unwrap_or_default();

        if webhook_creator_id == user_id_from_token {
            let _ = api_handle
                .send_request_json(
                    reqwest::Method::DELETE,
                    &format!("https://discord.com/api/v9/webhooks/{}", webhook_id),
                    None,
                    &token,
                    is_bearer,
                    None,
                )
                .await?;
            deleted_webhooks += 1;
        }
        let _ = window.emit(
            "webhook_progress",
            serde_json::json!({ "current": deleted_webhooks, "total": webhooks.len() }),
        );
    }

    op_manager.state.reset();
    let _ = window.emit("webhook_complete", ());
    Ok(())
}

#[tauri::command]
pub async fn open_discord_url_for_action(
    app_handle: AppHandle,
    action_type: String,
) -> Result<(), AppError> {
    let url = match action_type.as_str() {
        "account_deletion" => "https://discord.com/settings/account",
        "data_privacy" => "https://discord.com/settings/privacy",
        "gdpr_request" => {
            "https://support.discord.com/hc/en-us/articles/360004027692-Requesting-a-Copy-of-your-Data"
        }
        "support_portal" => "https://support.discord.com/hc/en-us/requests/new",
        _ => return Err(AppError::new("Unknown action", "invalid_action")),
    };

    app_handle
        .opener()
        .open_url(url, None::<String>)
        .map_err(|e| AppError::new(&e.to_string(), "external_link_error"))
}

#[tauri::command]
pub async fn open_external_link(app_handle: AppHandle, url: String) -> Result<(), AppError> {
    app_handle
        .opener()
        .open_url(url, None::<String>)
        .map_err(|e| AppError::new(&e.to_string(), "external_link_error"))
}

#[tauri::command]
pub async fn sanitize_media_metadata(
    app_handle: AppHandle,
    file_path: String,
) -> Result<(), AppError> {
    crate::core::forensics::metadata::MetadataStripper::strip_file(
        &app_handle,
        std::path::Path::new(&file_path),
    )
}

#[tauri::command]
pub async fn start_burner_protocol(app_handle: AppHandle) -> Result<(), AppError> {
    crate::core::forensics::burner::BurnerManager::initiate_burner_protocol(&app_handle)?;
    // After burning, we should probably exit or log out.
    // The command caller will handle UI state.
    Ok(())
}
