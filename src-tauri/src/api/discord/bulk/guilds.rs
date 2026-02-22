// src-tauri/src/api/discord/bulk/guilds.rs

use crate::api::rate_limiter::ApiHandle;
use crate::core::error::AppError;
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
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    for (i, guild_id) in guild_ids.iter().enumerate() {
        op_manager.state.wait_if_paused().await;
        if op_manager.state.should_abort.load(Ordering::SeqCst) {
            break;
        }

        let url = format!("https://discord.com/api/v10/users/@me/guilds/{}", guild_id);
        let _ = api_handle
            .send_request(reqwest::Method::DELETE, &url, None, &token, is_bearer)
            .await;
        let _ = window.emit("leave_progress", serde_json::json!({ "current": i + 1, "total": guild_ids.len(), "id": guild_id, "status": "severing" }));
    }
    op_manager.state.reset();
    let _ = window.emit("leave_complete", ());
    Ok(())
}
