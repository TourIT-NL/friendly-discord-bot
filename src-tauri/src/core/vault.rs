// src-tauri/src/core/vault.rs

use crate::core::error::AppError;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

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

impl Vault {
    const SERVICE_NAME: &'static str = "com.discordprivacy.util";

    fn get_fallback_path(app: &AppHandle, key: &str) -> Option<PathBuf> {
        app.path().app_local_data_dir().ok().map(|p| p.join(format!("{}.secure", key)))
    }

    fn write_fallback(app: &AppHandle, key: &str, value: &str) -> Result<(), AppError> {
        if let Some(path) = Self::get_fallback_path(app, key) {
            fs::write(path, value).map_err(AppError::from)?;
            Ok(())
        } else {
            Err(AppError { user_message: "Failed to resolve storage path.".into(), ..Default::default() })
        }
    }

    fn read_fallback(app: &AppHandle, key: &str) -> Result<String, AppError> {
        if let Some(path) = Self::get_fallback_path(app, key) {
            if path.exists() {
                return fs::read_to_string(path).map_err(AppError::from);
            }
        }
        Err(AppError { 
            user_message: "Credential not found in local storage.".into(), 
            error_code: "credentials_missing".into(),
            ..Default::default() 
        })
    }

    fn delete_fallback(app: &AppHandle, key: &str) -> Result<(), AppError> {
        if let Some(path) = Self::get_fallback_path(app, key) {
            if path.exists() {
                fs::remove_file(path).map_err(AppError::from)?;
            }
        }
        Ok(())
    }

    /// Persists a Discord identity. Tries Keyring first, falls back to File.
    pub fn save_identity(app: &AppHandle, identity: DiscordIdentity) -> Result<(), AppError> {
        let key = format!("account_{}", identity.id);
        let secret = serde_json::to_string(&identity)?;

        if let Ok(entry) = Entry::new(Self::SERVICE_NAME, &key) {
            if entry.set_password(&secret).is_err() {
                Self::write_fallback(app, &key, &secret)?;
            }
        } else {
            Self::write_fallback(app, &key, &secret)?;
        }
        
        // Track active account
        let active_key = "active_account";
        if let Ok(entry) = Entry::new(Self::SERVICE_NAME, active_key) {
            if entry.set_password(&identity.id).is_err() {
                Self::write_fallback(app, active_key, &identity.id)?;
            }
        } else {
            Self::write_fallback(app, active_key, &identity.id)?;
        }

        // Update index
        let index_key = "identity_index";
        let mut index = Self::get_credential(app, index_key)
            .and_then(|s| serde_json::from_str::<Vec<String>>(&s).map_err(AppError::from))
            .unwrap_or_default();
            
        if !index.contains(&identity.id) {
            index.push(identity.id.clone());
            let index_json = serde_json::to_string(&index)?;
            if let Ok(entry) = Entry::new(Self::SERVICE_NAME, index_key) {
                if entry.set_password(&index_json).is_err() {
                    Self::write_fallback(app, index_key, &index_json)?;
                }
            } else {
                Self::write_fallback(app, index_key, &index_json)?;
            }
        }
        Ok(())
    }

    /// Retrieves the currently active Discord token.
    pub fn get_active_token(app: &AppHandle) -> Result<(String, bool), AppError> {
        let id = match Entry::new(Self::SERVICE_NAME, "active_account").and_then(|e| e.get_password()) {
            Ok(p) => p,
            Err(_) => Self::read_fallback(app, "active_account").map_err(|_| AppError {
                user_message: "No active session found. Please login.".into(),
                error_code: "no_active_session".into(),
                ..Default::default()
            })?
        };

        let identity = Self::get_identity(app, &id)?;
        Ok((identity.token, identity.is_oauth))
    }

    /// Fetches a specific identity.
    pub fn get_identity(app: &AppHandle, id: &str) -> Result<DiscordIdentity, AppError> {
        let key = format!("account_{}", id);
        let secret = match Entry::new(Self::SERVICE_NAME, &key).and_then(|e| e.get_password()) {
            Ok(s) => s,
            Err(_) => Self::read_fallback(app, &key)?
        };
        Ok(serde_json::from_str(&secret)?)
    }

    /// Lists all Discord identities.
    pub fn list_identities(app: &AppHandle) -> Vec<DiscordIdentity> {
        let index_key = "identity_index";
        let index_str = Entry::new(Self::SERVICE_NAME, index_key)
            .and_then(|e| e.get_password())
            .or_else(|_| Self::read_fallback(app, index_key))
            .unwrap_or_default();

        let index: Vec<String> = serde_json::from_str(&index_str).unwrap_or_default();
        
        index.into_iter()
            .filter_map(|id| Self::get_identity(app, &id).ok())
            .collect()
    }

    /// Removes an identity.
    pub fn remove_identity(app: &AppHandle, id: &str) -> Result<(), AppError> {
        let key = format!("account_{}", id);
        let _ = Entry::new(Self::SERVICE_NAME, &key).map(|e| e.delete_credential());
        let _ = Self::delete_fallback(app, &key);

        // Remove from index
        let index_key = "identity_index";
        let index_str = Entry::new(Self::SERVICE_NAME, index_key)
            .and_then(|e| e.get_password())
            .or_else(|_| Self::read_fallback(app, index_key))
            .unwrap_or_default();

        if let Ok(mut index) = serde_json::from_str::<Vec<String>>(&index_str) {
            index.retain(|x| x != id);
            let new_index_json = serde_json::to_string(&index)?;
            
            if let Ok(entry) = Entry::new(Self::SERVICE_NAME, index_key) {
                if entry.set_password(&new_index_json).is_err() {
                    Self::write_fallback(app, index_key, &new_index_json)?;
                }
            } else {
                Self::write_fallback(app, index_key, &new_index_json)?;
            }
        }
        Ok(())
    }

    /// Stores a raw application credential (fallback enabled).
    pub fn set_credential(app: &AppHandle, key: &str, value: &str) -> Result<(), AppError> {
        if let Ok(entry) = Entry::new(Self::SERVICE_NAME, key) {
            if entry.set_password(value).is_err() {
                Self::write_fallback(app, key, value)?;
            }
        } else {
            Self::write_fallback(app, key, value)?;
        }
        Ok(())
    }

    /// Retrieves a raw application credential (fallback enabled).
    pub fn get_credential(app: &AppHandle, key: &str) -> Result<String, AppError> {
        Entry::new(Self::SERVICE_NAME, key)
            .and_then(|e| e.get_password())
            .or_else(|_| Self::read_fallback(app, key))
            .map_err(|_| AppError { 
                user_message: format!("Credential '{}' not found. Please complete Setup.", key), 
                error_code: "credentials_missing".into(),
                ..Default::default()
            })
    }
}
