// src-tauri/src/core/vault/identity.rs

use crate::core::error::AppError;
use crate::core::logger::Logger;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

/// Represents a validated Discord user identity stored in the Vault.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscordIdentity {
    /// Discord User ID (Snowflake).
    pub id: String,
    /// Discord username.
    pub username: String,
    /// Encrypted Discord API token (User Token or OAuth2 Access Token).
    pub token: String,
    /// Whether the token was obtained via OAuth2.
    pub is_oauth: bool,
}

/// Orchestrates the storage and retrieval of Discord identities.
/// Uses a combination of system keyring and encrypted disk fallback.
pub struct IdentityManager;

impl IdentityManager {
    const SERVICE_NAME: &'static str = "com.discordprivacy.util";

    /// Persistently saves an identity. Encrypts the payload before storage.
    /// Updates the global identity index to allow listing multiple accounts.
    pub fn save_identity(app: &AppHandle, identity: DiscordIdentity) -> Result<(), AppError> {
        let key = format!("account_{}", identity.id);
        let secret = serde_json::to_string(&identity)?;

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

        super::fallback::FallbackManager::write_fallback(app, &key, &secret)?;

        let active_key = "active_account";
        if let Ok(entry) = Entry::new(Self::SERVICE_NAME, active_key) {
            let _ = entry.set_password(&identity.id);
        }
        super::fallback::FallbackManager::write_fallback(app, active_key, &identity.id)?;

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
        super::fallback::FallbackManager::write_fallback(app, index_key, &index_json)?;

        Ok(())
    }

    /// Returns the identity of the currently logged-in user.
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
                super::fallback::FallbackManager::read_fallback(app, "active_account").map_err(
                    |_| AppError {
                        user_message: "No active session found. Please login.".into(),
                        error_code: "no_active_session".into(),
                        ..Default::default()
                    },
                )?
            }
        };

        Self::get_identity(app, &id)
    }

    /// Convenience method to retrieve the token and OAuth status of the active user.
    pub fn get_active_token(app: &AppHandle) -> Result<(String, bool), AppError> {
        let identity = Self::get_active_identity(app)?;
        Ok((identity.token, identity.is_oauth))
    }

    /// Retrieves a specific identity by Discord ID.
    pub fn get_identity(app: &AppHandle, id: &str) -> Result<DiscordIdentity, AppError> {
        let key = format!("account_{}", id);
        let secret = match Entry::new(Self::SERVICE_NAME, &key).and_then(|e| e.get_password()) {
            Ok(s) => s,
            Err(_) => super::fallback::FallbackManager::read_fallback(app, &key)?,
        };
        Ok(serde_json::from_str(&secret)?)
    }

    /// Enumerates all stored identities.
    pub fn list_identities(app: &AppHandle) -> Vec<DiscordIdentity> {
        let index_key = "identity_index";
        let index_str =
            match Entry::new(Self::SERVICE_NAME, index_key).and_then(|e| e.get_password()) {
                Ok(s) => s,
                Err(_) => super::fallback::FallbackManager::read_fallback(app, index_key)
                    .unwrap_or_else(|_| "[]".to_string()),
            };

        let index: Vec<String> = serde_json::from_str(&index_str).unwrap_or_default();
        index
            .into_iter()
            .filter_map(|id| Self::get_identity(app, &id).ok())
            .collect()
    }

    /// Completely removes an identity from both keyring and disk fallback.
    pub fn remove_identity(app: &AppHandle, id: &str) -> Result<(), AppError> {
        let key = format!("account_{}", id);
        let _ = Self::clear_keyring_entry(&key);
        let _ = super::fallback::FallbackManager::delete_fallback(app, &key);

        let index_key = "identity_index";
        let index_str =
            match Entry::new(Self::SERVICE_NAME, index_key).and_then(|e| e.get_password()) {
                Ok(s) => s,
                Err(_) => super::fallback::FallbackManager::read_fallback(app, index_key)
                    .unwrap_or_else(|_| "[]".to_string()),
            };

        if let Ok(mut index) = serde_json::from_str::<Vec<String>>(&index_str) {
            index.retain(|x| x != id);
            let new_index_json = serde_json::to_string(&index)?;

            let _ = Self::set_keyring_entry(index_key, &new_index_json);
            let _ =
                super::fallback::FallbackManager::write_fallback(app, index_key, &new_index_json);
        }
        Ok(())
    }

    /// Directly clears a credential from the OS keyring.
    pub fn clear_keyring_entry(key: &str) -> Result<(), AppError> {
        if let Ok(entry) = Entry::new(Self::SERVICE_NAME, key) {
            let _ = entry.delete_credential();
        }
        Ok(())
    }

    /// Directly sets a credential in the OS keyring.
    pub fn set_keyring_entry(key: &str, value: &str) -> Result<(), AppError> {
        if let Ok(entry) = Entry::new(Self::SERVICE_NAME, key) {
            let _ = entry.set_password(value);
        }
        Ok(())
    }
}
