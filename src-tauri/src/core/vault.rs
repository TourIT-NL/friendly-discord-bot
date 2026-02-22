// src-tauri/src/core/vault.rs

use crate::core::crypto::Crypto; // NEW
use crate::core::error::AppError;
use crate::core::logger::Logger;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::OnceLock; // Using std's OnceLock
use tauri::{AppHandle, Manager}; // Using std's Mutex

/// Represents a stored Discord identity, containing the unique user ID,
/// the current session token, and the authentication protocol used (OAuth vs User Token).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscordIdentity {
    pub id: String,
    pub username: String,
    pub token: String,
    pub is_oauth: bool,
}

/// The Vault is the primary security interface for sensitive data persistence.
/// It prioritizes the host OS keychain but falls back to encrypted local files
/// if the keyring service is unavailable or unstable.
pub struct Vault;

static KEY_CACHE: OnceLock<Mutex<Option<String>>> = OnceLock::new(); // NEW

impl Vault {
    const SERVICE_NAME: &'static str = "com.discordprivacy.util";
    const ENCRYPTION_KEY_SERVICE_NAME: &'static str = "com.discordprivacy.util.enc_key"; // NEW

    fn get_fallback_path(app: &AppHandle, key: &str) -> Option<PathBuf> {
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

    fn get_or_create_encryption_key(app: &AppHandle) -> Result<String, AppError> {
        // --- Step 1: Check in-memory cache ---
        if let Some(key_mutex) = KEY_CACHE.get() {
            let key_guard = key_mutex.lock().unwrap();
            if let Some(cached_key) = key_guard.as_ref() {
                return Ok(cached_key.clone());
            }
        }

        let key_name = Self::ENCRYPTION_KEY_SERVICE_NAME;

        // --- Step 2: Persistent retrieval attempt ---
        for attempt in 1..=3 {
            // A. Try Keyring
            if let Ok(entry) = Entry::new(Self::SERVICE_NAME, key_name)
                && let Ok(key) = entry.get_password()
            {
                let mutex = KEY_CACHE.get_or_init(|| Mutex::new(None));
                mutex.lock().unwrap().replace(key.clone());
                return Ok(key);
            }
            // B. Try Fallback File (Directly)
            if let Some(path) = Self::get_fallback_path(app, key_name)
                && path.exists()
                && let Ok(key) = fs::read_to_string(&path)
            {
                let mutex = KEY_CACHE.get_or_init(|| Mutex::new(None));
                mutex.lock().unwrap().replace(key.clone());
                return Ok(key);
            }
            std::thread::sleep(std::time::Duration::from_millis(100 * attempt));
        }

        // --- Step 3: Generate ONLY if absolutely nothing found ---
        Logger::warn(
            app,
            "[Vault] No existing encryption key found. Generating fresh identity key.",
            None,
        );
        let new_key = Crypto::generate_key();

        if let Ok(entry) = Entry::new(Self::SERVICE_NAME, key_name) {
            let _ = entry.set_password(&new_key);
        }

        if let Some(path) = Self::get_fallback_path(app, key_name) {
            let _ = fs::write(&path, &new_key);
        }

        let mutex = KEY_CACHE.get_or_init(|| Mutex::new(None));
        mutex.lock().unwrap().replace(new_key.clone());
        Ok(new_key)
    }
    fn write_fallback(app: &AppHandle, key: &str, value: &str) -> Result<(), AppError> {
        if let Some(path) = Self::get_fallback_path(app, key) {
            let enc_key = Self::get_or_create_encryption_key(app)?;
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

    fn read_fallback(app: &AppHandle, key: &str) -> Result<String, AppError> {
        if let Some(path) = Self::get_fallback_path(app, key) {
            if path.exists() {
                match fs::read_to_string(&path) {
                    Ok(encrypted_s) => {
                        let enc_key = Self::get_or_create_encryption_key(app)?;
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

    fn delete_fallback(app: &AppHandle, key: &str) -> Result<(), AppError> {
        if let Some(path) = Self::get_fallback_path(app, key)
            && path.exists()
        {
            fs::remove_file(path).map_err(AppError::from)?;
        }
        // For current implementation, the encryption key is global and not deleted with each item.
        // If specific keys were generated per item, they would be deleted here.
        Ok(())
    }

    /// Persists a Discord identity. Tries Keyring first, then always writes to File fallback.
    pub fn save_identity(app: &AppHandle, identity: DiscordIdentity) -> Result<(), AppError> {
        let key = format!("account_{}", identity.id);
        let secret = serde_json::to_string(&identity)?;

        // Strategy: Write Keyring -> Read Verify
        if let Ok(entry) = Entry::new(Self::SERVICE_NAME, &key) {
            if let Err(e) = entry.set_password(&secret) {
                Logger::warn(
                    app,
                    &format!(
                        "[Vault] Keyring write error for {}: {}. Using fallback.",
                        key, e
                    ),
                    None,
                );
            } else {
                // Verify write
                match entry.get_password() {
                    Ok(stored) if stored == secret => Logger::debug(
                        app,
                        &format!("[Vault] Verified Keyring storage for {}", key),
                        None,
                    ),
                    Ok(_) => Logger::warn(
                        app,
                        &format!("[Vault] Keyring verification mismatch for {}", key),
                        None,
                    ),
                    Err(e) => Logger::warn(
                        app,
                        &format!(
                            "[Vault] Keyring verification read failed for {}: {}",
                            key, e
                        ),
                        None,
                    ),
                }
            }
        }

        // Always write to fallback for maximum resilience
        Self::write_fallback(app, &key, &secret)?;

        // Track active account
        let active_key = "active_account";
        if let Ok(entry) = Entry::new(Self::SERVICE_NAME, active_key) {
            let _ = entry.set_password(&identity.id);
        }
        Self::write_fallback(app, active_key, &identity.id)?;

        // Update index
        let index_key = "identity_index";
        let mut index = Self::list_identities(app)
            .iter()
            .map(|i| i.id.clone())
            .collect::<Vec<_>>();
        if !index.contains(&identity.id) {
            index.push(identity.id.clone());
        }

        let index_json = serde_json::to_string(&index)?;
        if let Ok(entry) = Entry::new(Self::SERVICE_NAME, index_key) {
            let _ = entry.set_password(&index_json);
        }
        Self::write_fallback(app, index_key, &index_json)?;

        Ok(())
    }

    /// Retrieves the currently active Discord identity.
    pub fn get_active_identity(app: &AppHandle) -> Result<DiscordIdentity, AppError> {
        let id = match Entry::new(Self::SERVICE_NAME, "active_account")
            .and_then(|e| e.get_password())
        {
            Ok(p) => p,
            Err(e) => {
                Logger::debug(
                    app,
                    &format!(
                        "[Vault] Keyring read failed for active_account: {}. Checking fallback.",
                        e
                    ),
                    None,
                );
                Self::read_fallback(app, "active_account").map_err(|_| AppError {
                    user_message: "No active session found. Please login.".into(),
                    error_code: "no_active_session".into(),
                    ..Default::default()
                })?
            }
        };

        Self::get_identity(app, &id)
    }

    /// Retrieves the currently active Discord token.
    pub fn get_active_token(app: &AppHandle) -> Result<(String, bool), AppError> {
        let identity = Self::get_active_identity(app)?;
        Ok((identity.token, identity.is_oauth))
    }

    /// Fetches a specific identity.
    pub fn get_identity(app: &AppHandle, id: &str) -> Result<DiscordIdentity, AppError> {
        let key = format!("account_{}", id);
        let secret = match Entry::new(Self::SERVICE_NAME, &key).and_then(|e| e.get_password()) {
            Ok(s) => s,
            Err(_) => Self::read_fallback(app, &key)?,
        };
        Ok(serde_json::from_str(&secret)?)
    }

    /// Lists all Discord identities.
    pub fn list_identities(app: &AppHandle) -> Vec<DiscordIdentity> {
        let index_key = "identity_index";
        let index_str =
            match Entry::new(Self::SERVICE_NAME, index_key).and_then(|e| e.get_password()) {
                Ok(s) => s,
                Err(_) => Self::read_fallback(app, index_key).unwrap_or_else(|_| "[]".to_string()),
            };

        let index: Vec<String> = serde_json::from_str(&index_str).unwrap_or_default();
        index
            .into_iter()
            .filter_map(|id| Self::get_identity(app, &id).ok())
            .collect()
    }

    /// Removes an identity.
    pub fn remove_identity(app: &AppHandle, id: &str) -> Result<(), AppError> {
        let key = format!("account_{}", id);
        if let Ok(entry) = Entry::new(Self::SERVICE_NAME, &key) {
            let _ = entry.delete_credential();
        }
        let _ = Self::delete_fallback(app, &key);

        // Remove from index
        let index_key = "identity_index";
        let index_str =
            match Entry::new(Self::SERVICE_NAME, index_key).and_then(|e| e.get_password()) {
                Ok(s) => s,
                Err(_) => Self::read_fallback(app, index_key).unwrap_or_else(|_| "[]".to_string()),
            };

        if let Ok(mut index) = serde_json::from_str::<Vec<String>>(&index_str) {
            index.retain(|x| x != id);
            let new_index_json = serde_json::to_string(&index)?;

            if let Ok(entry) = Entry::new(Self::SERVICE_NAME, index_key) {
                let _ = entry.set_password(&new_index_json);
            }
            Self::write_fallback(app, index_key, &new_index_json)?;
        }
        Ok(())
    }

    /// Stores a raw application credential (fallback enabled).
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
        Self::write_fallback(app, key, value)?;
        Ok(())
    }

    /// Retrieves a raw application credential (fallback enabled).
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
                    Self::read_fallback(app, key)
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
                Self::read_fallback(app, key)
            }
        };

        result.map_err(|original_err| {
            if original_err.error_code == "credentials_missing" {
                AppError {
                    user_message: format!("Credential '{}' not found. Please complete Setup.", key),
                    error_code: "vault_credentials_missing".into(), // Specific code for missing
                    technical_details: original_err.technical_details,
                }
            } else {
                original_err // Propagate other errors as they are
            }
        })
    }
}
