// src-tauri/src/core/vault/commands.rs

use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::vault::encryption::EncryptionManager;
use crate::core::vault::state::VaultState;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn is_vault_locked(
    app_handle: AppHandle,
    state: State<'_, VaultState>,
) -> Result<bool, AppError> {
    if EncryptionManager::has_master_password(&app_handle)? {
        let key_guard = state.encryption_key.lock().unwrap();
        Ok(key_guard.is_none())
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub async fn has_master_password(app_handle: AppHandle) -> Result<bool, AppError> {
    EncryptionManager::has_master_password(&app_handle)
}

#[tauri::command]
pub async fn set_master_password(
    app_handle: AppHandle,
    state: State<'_, VaultState>,
    password: Option<String>,
) -> Result<(), AppError> {
    EncryptionManager::set_master_password(&app_handle, password.as_deref())?;

    // After setting or clearing, we need to update the in-memory key
    let mut key_guard = state.encryption_key.lock().unwrap();
    if password.is_some() {
        // If we just set it, it's effectively "locked" until they provide it again
        *key_guard = None;
    } else {
        // If we cleared it, the key is now plaintext in the keyring, so we can re-load it
        *key_guard = Some(zeroize::Zeroizing::new(
            EncryptionManager::get_or_create_encryption_key(&app_handle)?,
        ));
    }

    Logger::info(&app_handle, "[Vault] Master password status updated", None);
    Ok(())
}

#[tauri::command]
pub async fn has_biometric_support() -> Result<bool, AppError> {
    #[cfg(target_os = "windows")]
    {
        // Simple check via windows-sys if possible, or just return false if not implemented
        Ok(true) // Placeholder for Windows Hello integration
    }
    #[cfg(not(target_os = "windows"))]
    Ok(false)
}

#[tauri::command]
pub async fn unlock_vault(
    app_handle: AppHandle,
    state: State<'_, VaultState>,
    password: String,
) -> Result<(), AppError> {
    let key = EncryptionManager::unlock_with_password(&app_handle, &password)?;
    let mut key_guard = state.encryption_key.lock().unwrap();
    *key_guard = Some(zeroize::Zeroizing::new(key));

    Logger::info(&app_handle, "[Vault] Vault unlocked successfully", None);
    Ok(())
}
