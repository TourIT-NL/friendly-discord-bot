// src-tauri/src/core/vault/credential.rs

use crate::core::error::AppError;
use crate::core::logger::Logger;
use keyring::Entry;
use tauri::AppHandle;

pub struct CredentialManager;

impl CredentialManager {
    const SERVICE_NAME: &'static str = "com.discordprivacy.util";

    pub fn set_credential(app: &AppHandle, key: &str, value: &str) -> Result<(), AppError> {
        if let Ok(entry) = Entry::new(Self::SERVICE_NAME, key) {
            if let Err(e) = entry.set_password(value) {
                Logger::warn(
                    app,
                    &format!(
                        "[Vault] Credential keyring write failed: {}. Using fallback.",
                        e
                    ),
                    None,
                );
            } else {
                Logger::debug(app, &format!("[Vault] Saved {} to Keyring", key), None);
            }
        }
        super::fallback::FallbackManager::write_fallback(app, key, value)?;
        Ok(())
    }

    pub fn get_credential(app: &AppHandle, key: &str) -> Result<String, AppError> {
        let result = match Entry::new(Self::SERVICE_NAME, key) {
            Ok(entry) => match entry.get_password() {
                Ok(p) => Ok(p),
                Err(e) => {
                    Logger::debug(
                        app,
                        &format!(
                            "[Vault] Keyring read failed for {}: {}. Checking fallback.",
                            key, e
                        ),
                        None,
                    );
                    super::fallback::FallbackManager::read_fallback(app, key)
                }
            },
            Err(e) => {
                Logger::debug(
                    app,
                    &format!(
                        "[Vault] Keyring entry failed for {}: {}. Checking fallback.",
                        key, e
                    ),
                    None,
                );
                super::fallback::FallbackManager::read_fallback(app, key)
            }
        };

        result.map_err(|original_err| {
            if original_err.error_code == "credentials_missing" {
                AppError {
                    user_message: format!("Credential '{}' not found. Please complete Setup.", key),
                    error_code: "vault_credentials_missing".into(),
                    technical_details: original_err.technical_details,
                }
            } else {
                original_err
            }
        })
    }

    pub fn remove_credential(app: &AppHandle, key: &str) -> Result<(), AppError> {
        if let Ok(entry) = Entry::new(Self::SERVICE_NAME, key) {
            let _ = entry.delete_credential();
        }
        let _ = super::fallback::FallbackManager::delete_fallback(app, key);
        Logger::debug(app, &format!("[Vault] Removed {} from storage", key), None);
        Ok(())
    }
}
