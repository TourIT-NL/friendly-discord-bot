// src-tauri/src/core/vault/fallback.rs

use crate::core::crypto::Crypto;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub struct FallbackManager;

impl FallbackManager {
    pub fn get_fallback_path(app: &AppHandle, key: &str) -> Option<PathBuf> {
        match app.path().app_local_data_dir() {
            Ok(p) => {
                let secure_path = p.join(format!("{}.secure", key));
                Some(secure_path)
            }
            Err(e) => {
                Logger::error(
                    app,
                    &format!("[Vault] Failed to resolve data dir: {}", e),
                    None,
                );
                None
            }
        }
    }

    pub fn write_fallback(app: &AppHandle, key: &str, value: &str) -> Result<(), AppError> {
        if let Some(path) = Self::get_fallback_path(app, key) {
            let enc_key = super::encryption::EncryptionManager::get_or_create_encryption_key(app)?;
            let encrypted_value = Crypto::encrypt(&enc_key, value)?;

            if let Err(e) = fs::write(&path, encrypted_value) {
                Logger::error(
                    app,
                    &format!("[Vault] File write failed for {}: {}", key, e),
                    None,
                );
                return Err(AppError::from(e));
            }
            Logger::debug(
                app,
                &format!("[Vault] Saved {} to disk fallback (encrypted)", key),
                None,
            );
            Ok(())
        } else {
            Err(AppError {
                user_message: "Failed to resolve storage path.".into(),
                ..Default::default()
            })
        }
    }

    pub fn read_fallback(app: &AppHandle, key: &str) -> Result<String, AppError> {
        if let Some(path) = Self::get_fallback_path(app, key) {
            if path.exists() {
                match fs::read_to_string(&path) {
                    Ok(encrypted_s) => {
                        let enc_key = super::encryption::EncryptionManager::get_or_create_encryption_key(app)?;
                        return Crypto::decrypt(&enc_key, &encrypted_s);
                    }
                    Err(e) => {
                        Logger::error(
                            app,
                            &format!("[Vault] File read failed for {}: {}", key, e),
                            None,
                        );
                        return Err(AppError::from(e));
                    }
                }
            } else {
                Logger::warn(app, &format!("[Vault] File not found: {:?}", path), None);
            }
        }
        Err(AppError {
            user_message: "Credential not found in local storage.".into(),
            error_code: "credentials_missing".into(),
            ..Default::default()
        })
    }

    pub fn delete_fallback(app: &AppHandle, key: &str) -> Result<(), AppError> {
        if let Some(path) = Self::get_fallback_path(app, key)
            && path.exists()
        {
            fs::remove_file(path).map_err(AppError::from)?;
        }
        Ok(())
    }
}
