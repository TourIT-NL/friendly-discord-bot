// src-tauri/src/api/discord/bulk/guilds.rs

use crate::api::rate_limiter::ApiHandle;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::op_manager::OperationManager;
use crate::core::vault::Vault;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager, Window};

#[tauri::command]
pub async fn bulk_leave_guilds(
    app_handle: AppHandle,
    window: Window,
    guild_ids: Vec<String>,
) -> Result<(), AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.prepare(); // Ensure clean state
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    Logger::info(
        &app_handle,
        &format!("[OP] Bulk leave initialized for {} guilds", guild_ids.len()),
        None,
    );

    for (i, guild_id) in guild_ids.iter().enumerate() {
        op_manager.state.wait_if_paused().await;
        if op_manager.state.should_abort.load(Ordering::SeqCst) {
            Logger::info(&app_handle, "[OP] Bulk leave aborted by user", None);
            break;
        }

        let url = format!("https://discord.com/api/v9/users/@me/guilds/{}", guild_id);
        match api_handle
            .send_request(reqwest::Method::DELETE, &url, None, &token, is_bearer)
            .await
        {
            Ok(_) => {
                // Check abort again before emitting progress
                if op_manager.state.should_abort.load(Ordering::SeqCst) {
                    break;
                }
                let _ = window.emit(
                    "leave_progress",
                    serde_json::json!({
                        "current": i + 1,
                        "total": guild_ids.len(),
                        "id": guild_id,
                        "status": "severed"
                    }),
                );
            }
            Err(e) => {
                Logger::warn(
                    &app_handle,
                    &format!(
                        "[OP] Failed to leave guild {}: {}",
                        guild_id, e.user_message
                    ),
                    None,
                );
            }
        }
    }
    op_manager.state.reset();
    let _ = window.emit("leave_complete", ());
    Logger::info(&app_handle, "[OP] Bulk leave operation completed", None);
    Ok(())
}
