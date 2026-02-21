// src-tauri/src/core/vault.rs

use tauri::AppHandle;
use keyring::Entry;
use serde::{Serialize, Deserialize};
use crate::core::error::AppError;

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
/// It utilizes the host OS keychain (Windows Credential Manager, macOS Keychain, or Secret Service)
/// to ensure that Discord tokens and application credentials never reside in plain text on the disk.
pub struct Vault;

impl Vault {
    const SERVICE_NAME: &'static str = "com.discordprivacy.util";

    /// Persists a Discord identity to the secure OS vault.
    pub fn save_identity(_app: &AppHandle, identity: DiscordIdentity) -> Result<(), AppError> {
        let entry = Entry::new(Self::SERVICE_NAME, &format!("account_{}", identity.id))?;
        let secret = serde_json::to_string(&identity)?;
        entry.set_password(&secret)?;
        
        // Track this as the most recent 'active' account
        let active_entry = Entry::new(Self::SERVICE_NAME, "active_account")?;
        active_entry.set_password(&identity.id)?;

        // Update identity index
        let index_entry = Entry::new(Self::SERVICE_NAME, "identity_index")?;
        let mut index: Vec<String> = match index_entry.get_password() {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => Vec::new(),
        };
        if !index.contains(&identity.id) {
            index.push(identity.id.clone());
            index_entry.set_password(&serde_json::to_string(&index)?)?;
        }
        Ok(())
    }

    /// Retrieves the currently active Discord token and its type.
    pub fn get_active_token(_app: &AppHandle) -> Result<(String, bool), AppError> {
        let active_entry = Entry::new(Self::SERVICE_NAME, "active_account")?;
        let id = active_entry.get_password().map_err(|_| AppError { 
            user_message: "No active session found. Please login.".into(), 
            error_code: "no_active_session".into(),
            ..Default::default()
        })?;
        
        let identity = Self::get_identity(_app, &id)?;
        Ok((identity.token, identity.is_oauth))
    }

    /// Fetches a specific identity from the vault by its Discord ID.
    pub fn get_identity(_app: &AppHandle, id: &str) -> Result<DiscordIdentity, AppError> {
        let entry = Entry::new(Self::SERVICE_NAME, &format!("account_{}", id))?;
        let secret = entry.get_password()?;
        Ok(serde_json::from_str(&secret)?)
    }

    /// Lists all Discord identities currently stored in the system vault.
    pub fn list_identities(_app: &AppHandle) -> Vec<DiscordIdentity> {
        let index_entry = Entry::new(Self::SERVICE_NAME, "identity_index").ok();
        let index: Vec<String> = index_entry.and_then(|e| e.get_password().ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        
        let results: Vec<DiscordIdentity> = index.into_iter()
            .filter_map(|id| Self::get_identity(_app, &id).ok())
            .collect();

        // Fallback logic
        if results.is_empty() {
            if let Ok((_token, _is_oauth)) = Self::get_active_token(_app) {
                // Recovery possible on next successful validation
            }
        }
        results
    }

    /// Removes an identity from the vault, permanently destroying the token link.
    pub fn remove_identity(_app: &AppHandle, id: &str) -> Result<(), AppError> {
        let entry = Entry::new(Self::SERVICE_NAME, &format!("account_{}", id))?;
        let _ = entry.delete_credential();

        // Remove from index
        let index_entry = Entry::new(Self::SERVICE_NAME, "identity_index")?;
        if let Ok(s) = index_entry.get_password() {
            if let Ok(mut index) = serde_json::from_str::<Vec<String>>(&s) {
                index.retain(|x| x != id);
                let _ = index_entry.set_password(&serde_json::to_string(&index)?);
            }
        }
        Ok(())
    }

    /// Stores a raw application credential (like Client ID or Secret).
    pub fn set_credential(_app: &AppHandle, key: &str, value: &str) -> Result<(), AppError> {
        let entry = Entry::new(Self::SERVICE_NAME, key)?;
        entry.set_password(value)?;
        Ok(())
    }

    /// Retrieves a raw application credential.
    pub fn get_credential(_app: &AppHandle, key: &str) -> Result<String, AppError> {
        let entry = Entry::new(Self::SERVICE_NAME, key)?;
        entry.get_password().map_err(|_| AppError { 
            user_message: format!("Credential '{}' not found. Please complete Setup.", key), 
            error_code: "credentials_missing".into(),
            ..Default::default()
        })
    }
}
