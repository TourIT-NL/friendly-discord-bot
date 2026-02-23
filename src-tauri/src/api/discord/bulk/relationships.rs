// src-tauri/src/api/discord/bulk/relationships.rs

use crate::api::rate_limiter::ApiHandle;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::op_manager::OperationManager;
use crate::core::vault::Vault;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager, Window};

#[tauri::command]
pub async fn bulk_remove_relationships(
    app_handle: AppHandle,
    window: Window,
    user_ids: Vec<String>,
) -> Result<(), AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.prepare(); // Ensure clean state
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    Logger::info(
        &app_handle,
        &format!(
            "[OP] Bulk relationship removal initialized for {} users",
            user_ids.len()
        ),
        None,
    );

    for (i, user_id) in user_ids.iter().enumerate() {
        op_manager.state.wait_if_paused().await;
        if op_manager.state.should_abort.load(Ordering::SeqCst) {
            Logger::info(&app_handle, "[OP] Bulk relationship removal aborted", None);
            break;
        }

        let url = format!(
            "https://discord.com/api/v9/users/@me/relationships/{}",
            user_id
        );
        match api_handle
            .send_request(reqwest::Method::DELETE, &url, None, &token, is_bearer)
            .await
        {
            Ok(_) => {
                if op_manager.state.should_abort.load(Ordering::SeqCst) {
                    break;
                }
                let _ = window.emit(
                    "relationship_progress",
                    serde_json::json!({
                        "current": i + 1,
                        "total": user_ids.len(),
                        "id": user_id,
                        "status": "removed"
                    }),
                );
            }
            Err(e) => {
                Logger::warn(
                    &app_handle,
                    &format!(
                        "[OP] Failed to remove relationship {}: {}",
                        user_id, e.user_message
                    ),
                    None,
                );
            }
        }
    }
    op_manager.state.reset();
    let _ = window.emit("relationship_complete", ());
    Logger::info(
        &app_handle,
        "[OP] Bulk relationship removal completed",
        None,
    );
    Ok(())
}
