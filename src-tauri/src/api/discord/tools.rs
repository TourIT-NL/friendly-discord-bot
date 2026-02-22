// src-tauri/src/api/discord/tools.rs

use crate::api::rate_limiter::ApiHandle;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::op_manager::OperationManager;
use crate::core::vault::Vault;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, Window};

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
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    if is_bearer {
        return Err(AppError {
            user_message: "Audit Log Burial restricted in Official Gate.".into(),
            ..Default::default()
        });
    }

    Logger::info(
        &app_handle,
        &format!("[AUDIT] Starting burial sequence in guild {}", guild_id),
        None,
    );

    let original_channel_value = api_handle
        .send_request(
            reqwest::Method::GET,
            &format!("https://discord.com/api/v10/channels/{}", channel_id),
            None,
            &token,
            is_bearer,
        )
        .await?; // Will return serde_json::Value if successful

    let original_channel_name = original_channel_value["name"]
        .as_str()
        .unwrap_or("general")
        .to_string();

    for i in 0..10 {
        op_manager.state.wait_if_paused().await;
        if op_manager.state.should_abort.load(Ordering::SeqCst) {
            break;
        }

        let new_name = format!("{}-temp-{}", original_channel_name, i);
        Logger::debug(
            &app_handle,
            &format!("[AUDIT] Phase {}: cyclic node rename", i),
            None,
        );
        let _ = api_handle
            .send_request(
                reqwest::Method::PATCH,
                &format!("https://discord.com/api/v10/channels/{}", channel_id),
                Some(serde_json::json!({ "name": new_name })),
                &token,
                is_bearer,
            )
            .await;

        let _ = window.emit("audit_log_progress", serde_json::json!({ "current": i + 1, "total": 20, "status": format!("Burying node data phase {}", i) }));
        tokio::time::sleep(Duration::from_millis(500)).await;

        let _ = api_handle
            .send_request(
                reqwest::Method::PATCH,
                &format!("https://discord.com/api/v10/channels/{}", channel_id),
                Some(serde_json::json!({ "name": original_channel_name })),
                &token,
                is_bearer,
            )
            .await;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    op_manager.state.reset();
    let _ = window.emit("audit_log_complete", ());
    Logger::info(&app_handle, "[AUDIT] Burial protocol finalized.", None);
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
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    if is_bearer {
        op_manager.state.reset();
        return Err(AppError {
            user_message: "Webhook Ghosting restricted in Official Gate.".into(),
            ..Default::default()
        });
    }

    Logger::info(
        &app_handle,
        &format!("[WEBHOOK] Ghosting identity hooks in node {}", guild_id),
        None,
    );

    let webhooks_value = api_handle
        .send_request(
            reqwest::Method::GET,
            &format!("https://discord.com/api/v10/guilds/{}/webhooks", guild_id),
            None,
            &token,
            is_bearer,
        )
        .await?; // Will return serde_json::Value if successful

    let webhooks: Vec<serde_json::Value> = serde_json::from_value(webhooks_value).map_err(AppError::from)?;

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
            Logger::debug(
                &app_handle,
                &format!("[WEBHOOK] Nullifying hook {}", webhook_id),
                None,
            );
            let _ = api_handle
                .send_request(
                    reqwest::Method::DELETE,
                    &format!("https://discord.com/api/v10/webhooks/{}", webhook_id),
                    None,
                    &token,
                    is_bearer,
                )
                .await;
            deleted_webhooks += 1;
        }
        let _ = window.emit("webhook_progress", serde_json::json!({ "current": deleted_webhooks, "total": webhooks.len(), "status": "Ghosting active" }));
    }

    op_manager.state.reset();
    let _ = window.emit("webhook_complete", ());
    Logger::info(
        &app_handle,
        &format!(
            "[WEBHOOK] Ghosting complete. Nullified {} identity hooks",
            deleted_webhooks
        ),
        None,
    );
    Ok(())
}
