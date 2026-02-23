// src-tauri/src/core/cleanup.rs

use super::error::AppError;
use super::logger::Logger;
use super::vault::Vault;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager; // Added for app_handle.path() // Added for PathBuf

#[tauri::command]
pub async fn clear_all_app_data(app_handle: AppHandle) -> Result<(), AppError> {
    Logger::info(
        &app_handle,
        "[Cleanup] Starting full application reset...",
        None,
    );

    // 1. Clear Vault data (identities and credentials)
    Vault::clear_all_data(&app_handle)?;

    // 2. Attempt to delete old log files (but skip the current one if possible)
    if let Ok(app_local_data_dir) = app_handle.path().app_local_data_dir() {
        let log_dir: PathBuf = app_local_data_dir;

        match tokio::fs::read_dir(&log_dir).await {
            Ok(mut entries) => {
                while let Some(entry) = entries.next_entry().await.transpose() {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_file()
                            && path
                                .file_name()
                                .and_then(|name| name.to_str())
                                .is_some_and(|name| name.starts_with("app.log"))
                        {
                            // Try to delete. If it fails (e.g. locked), just log and continue.
                            if let Err(e) = tokio::fs::remove_file(&path).await {
                                Logger::debug(
                                    &app_handle,
                                    &format!(
                                        "[Cleanup] Could not delete log file {} (likely in use): {}",
                                        path.display(),
                                        e
                                    ),
                                    None,
                                );
                            } else {
                                Logger::debug(
                                    &app_handle,
                                    &format!("[Cleanup] Deleted log file: {}", path.display()),
                                    None,
                                );
                            }
                        }
                    }
                }
            }
            Err(e) => {
                Logger::warn(
                    &app_handle,
                    &format!("[Cleanup] Could not access log directory: {}", e),
                    None,
                );
            }
        }
    }

    Logger::info(
        &app_handle,
        "[Cleanup] Application reset complete. Session terminated.",
        None,
    );
    Ok(())
}
