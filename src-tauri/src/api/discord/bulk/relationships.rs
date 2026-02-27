// src-tauri/src/api/discord/bulk/relationships.rs

use crate::api::rate_limiter::ApiHandle;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::op_manager::OperationManager;
use crate::core::vault::Vault;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager, Window};

#[tauri::command]
pub async fn bulk_cleanup_relationships(
    app_handle: AppHandle,
    window: Window,
    user_ids: Vec<String>,
    action: String, // "remove", "block", "ignore", "cancel"
) -> Result<(), AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>().inner().clone();
    let op_manager_state = app_handle.state::<OperationManager>().state.clone();
    op_manager_state.prepare();
    op_manager_state.is_running.store(true, Ordering::SeqCst);

    Logger::info(
        &app_handle,
        &format!(
            "[OP] Bulk relationship {} initialized for {} users (CONCURRENT)",
            action,
            user_ids.len()
        ),
        None,
    );

    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(user_ids.len());

    for (i, user_id) in user_ids.iter().cloned().enumerate() {
        let window_clone = window.clone();
        let token_clone = token.clone();
        let api_handle_clone = api_handle.clone();
        let op_manager_state_clone = op_manager_state.clone();
        let action_clone = action.clone();
        let total = user_ids.len();
        let tx_clone = tx.clone();

        tauri::async_runtime::spawn(async move {
            op_manager_state_clone.wait_if_paused().await;
            if op_manager_state_clone.should_abort.load(Ordering::SeqCst) {
                return;
            }

            let url = format!(
                "https://discord.com/api/v9/users/@me/relationships/{}",
                user_id
            );
            let method = if action_clone == "block" {
                reqwest::Method::PUT
            } else {
                reqwest::Method::DELETE
            };
            let body = if action_clone == "block" {
                Some(serde_json::json!({ "type": 2 }))
            } else {
                None
            };

            if api_handle_clone
                .send_request_json(method, &url, body, &token_clone, is_bearer, None)
                .await
                .is_ok()
            {
                let _ = window_clone.emit(
                    "relationship_progress",
                    serde_json::json!({
                        "current": i + 1,
                        "total": total,
                        "id": user_id,
                        "status": format!("{}d", action_clone)
                    }),
                );
                let _ = tx_clone.send(()).await;
            }
        });
    }

    drop(tx);
    while rx.recv().await.is_some() {}

    op_manager_state.reset();
    let _ = window.emit("relationship_complete", ());
    Ok(())
}
