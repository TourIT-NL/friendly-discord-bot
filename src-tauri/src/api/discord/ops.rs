// src-tauri/src/api/discord/ops.rs

use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::op_manager::OperationManager;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Manager, Window};

#[tauri::command]
pub async fn nuclear_wipe(app_handle: AppHandle, window: Window) -> Result<(), AppError> {
    Logger::info(
        &app_handle,
        "[NUCLEAR] INITIATING COMPLETE DIGITAL FOOTPRINT SANITIZATION",
        None,
    );

    // 1. Ghost Profile
    super::privacy::ghost_profile(app_handle.clone()).await?;

    // 2. Max Privacy Settings
    super::privacy::set_max_privacy_settings(app_handle.clone()).await?;

    // 3. Relationships cleanup (Remove all friends)
    let relationships = super::sync::fetch_relationships(app_handle.clone()).await?;
    let friend_ids: Vec<String> = relationships.iter().map(|r| r.id.clone()).collect();
    if !friend_ids.is_empty() {
        super::bulk::relationships::bulk_cleanup_relationships(
            app_handle.clone(),
            window.clone(),
            friend_ids,
            "remove".to_string(),
        )
        .await?;
    }

    // 4. Leave all guilds (except owned)
    let guilds = super::sync::fetch_guilds(app_handle.clone()).await?;
    let guild_ids: Vec<String> = guilds.iter().map(|g| g.id.clone()).collect();
    if !guild_ids.is_empty() {
        super::bulk::guilds::bulk_leave_guilds(app_handle.clone(), window.clone(), guild_ids)
            .await?;
    }

    Logger::info(
        &app_handle,
        "[NUCLEAR] Sanitization complete. User is now a ghost.",
        None,
    );
    Ok(())
}

#[tauri::command]
pub async fn pause_operation(app_handle: AppHandle) -> Result<(), AppError> {
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.is_paused.store(true, Ordering::SeqCst);
    Logger::info(&app_handle, "[OP] Operation paused by user", None);
    Ok(())
}

#[tauri::command]
pub async fn resume_operation(app_handle: AppHandle) -> Result<(), AppError> {
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.is_paused.store(false, Ordering::SeqCst);
    op_manager.state.pause_notifier.notify_waiters();
    Logger::info(&app_handle, "[OP] Operation resumed", None);
    Ok(())
}

#[tauri::command]
pub async fn abort_operation(app_handle: AppHandle) -> Result<(), AppError> {
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.should_abort.store(true, Ordering::SeqCst);
    op_manager.state.pause_notifier.notify_waiters();
    Logger::warn(&app_handle, "[OP] Abort signal sent", None);
    Ok(())
}

#[tauri::command]
pub async fn get_operation_status(
    app_handle: AppHandle,
) -> Result<super::types::OperationStatus, AppError> {
    let op_manager = app_handle.state::<OperationManager>();
    Ok(super::types::OperationStatus {
        is_running: op_manager.state.is_running.load(Ordering::SeqCst),
        is_paused: op_manager.state.is_paused.load(Ordering::SeqCst),
        should_abort: op_manager.state.should_abort.load(Ordering::SeqCst),
    })
}
