// src-tauri/src/api/discord/ops.rs

use super::types::OperationStatus;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::op_manager::OperationManager;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn pause_operation(app_handle: AppHandle) -> Result<(), AppError> {
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.is_paused.store(true, Ordering::SeqCst);
    Logger::warn(&app_handle, "[OP] Execution loop PAUSED", None);
    Ok(())
}

#[tauri::command]
pub async fn resume_operation(app_handle: AppHandle) -> Result<(), AppError> {
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.is_paused.store(false, Ordering::SeqCst);
    op_manager.state.pause_notifier.notify_waiters();
    Logger::info(&app_handle, "[OP] Execution loop RESUMED", None);
    Ok(())
}

#[tauri::command]
pub async fn abort_operation(app_handle: AppHandle) -> Result<(), AppError> {
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.should_abort.store(true, Ordering::SeqCst);
    op_manager.state.is_paused.store(false, Ordering::SeqCst);
    op_manager.state.pause_notifier.notify_waiters();
    Logger::error(
        &app_handle,
        "[OP] ABORT command received. Terminating loops...",
        None,
    );
    Ok(())
}

#[tauri::command]
pub async fn get_operation_status(app_handle: AppHandle) -> Result<OperationStatus, AppError> {
    let op_manager = app_handle.state::<OperationManager>();
    Ok(OperationStatus {
        is_running: op_manager.state.is_running.load(Ordering::SeqCst),
        is_paused: op_manager.state.is_paused.load(Ordering::SeqCst),
        should_abort: op_manager.state.should_abort.load(Ordering::SeqCst),
    })
}
