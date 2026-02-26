// src-tauri/src/api/discord/bulk/guilds.rs

use crate::api::rate_limiter::{ApiHandle, types::ApiResponseContent};
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
    let api_handle = app_handle.state::<ApiHandle>().inner().clone();
    let op_manager_state = app_handle.state::<OperationManager>().state.clone();
    op_manager_state.prepare();
    op_manager_state.is_running.store(true, Ordering::SeqCst);

    let guilds_json = api_handle
        .send_request_json(
            reqwest::Method::GET,
            "https://discord.com/api/v9/users/@me/guilds",
            None,
            &token,
            is_bearer,
            None,
        )
        .await?;

    let guilds: Vec<crate::api::discord::types::Guild> =
        serde_json::from_value(guilds_json).map_err(AppError::from)?;

    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(guild_ids.len());

    for (i, guild_id) in guild_ids.iter().cloned().enumerate() {
        let app_handle_clone = app_handle.clone();
        let window_clone = window.clone();
        let token_clone = token.clone();
        let api_handle_clone = api_handle.clone();
        let op_state = op_manager_state.clone();
        let tx_clone = tx.clone();
        let total = guild_ids.len();

        let is_owner = guilds
            .iter()
            .find(|g| g.id == guild_id)
            .map(|g| g.owner)
            .unwrap_or(false);

        tauri::async_runtime::spawn(async move {
            op_state.wait_if_paused().await;
            if op_state.should_abort.load(Ordering::SeqCst) || is_owner {
                return;
            }

            let url = format!("https://discord.com/api/v9/users/@me/guilds/{}", guild_id);
            if api_handle_clone
                .send_request(
                    reqwest::Method::DELETE,
                    &url,
                    None,
                    &token_clone,
                    is_bearer,
                    false,
                    None,
                    None,
                    None,
                    None,
                )
                .await
                .is_ok()
            {
                let _ = window_clone.emit("leave_progress", serde_json::json!({ "current": i + 1, "total": total, "id": guild_id, "status": "severed" }));
                let _ = tx_clone.send(()).await;
            }
        });
    }

    drop(tx);
    while rx.recv().await.is_some() {}
    op_manager_state.reset();
    let _ = window.emit("leave_complete", ());
    Ok(())
}
