// src-tauri/src/core/vault/mod.rs

pub mod commands;
pub mod credential;
pub mod encryption;
pub mod fallback;
pub mod identity;
pub mod state;

pub use identity::DiscordIdentity;
pub use state::VaultState;

use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::vault::fallback::FallbackManager; // Added import
use tauri::AppHandle;

pub struct Vault;

impl Vault {
    /// Persistently saves a Discord user identity to secure storage.
    /// Encrypts the identity using the Vault's master key before storing.
    pub fn save_identity(app: &AppHandle, identity: DiscordIdentity) -> Result<(), AppError> {
        identity::IdentityManager::save_identity(app, identity)
    }

    /// Retrieves the currently active user identity.
    /// Returns an error if no user is logged in.
    pub fn get_active_identity(app: &AppHandle) -> Result<DiscordIdentity, AppError> {
        identity::IdentityManager::get_active_identity(app)
    }

    /// Convenience method to get the active token and its OAuth status.
    pub fn get_active_token(app: &AppHandle) -> Result<(String, bool), AppError> {
        identity::IdentityManager::get_active_token(app)
    }

    #[allow(dead_code)]
    /// Retrieves a specific identity by Discord User ID.
    pub fn get_identity(app: &AppHandle, id: &str) -> Result<DiscordIdentity, AppError> {
        identity::IdentityManager::get_identity(app, id)
    }

    /// Returns a list of all stored identities.
    pub fn list_identities(app: &AppHandle) -> Vec<DiscordIdentity> {
        identity::IdentityManager::list_identities(app)
    }

    /// Removes an identity from secure storage and the index.
    pub fn remove_identity(app: &AppHandle, id: &str) -> Result<(), AppError> {
        identity::IdentityManager::remove_identity(app, id)
    }

    /// Saves a generic credential (e.g., client_id, proxy_url) to secure storage.
    pub fn set_credential(app: &AppHandle, key: &str, value: &str) -> Result<(), AppError> {
        credential::CredentialManager::set_credential(app, key, value)
    }

    /// Retrieves a generic credential by key.
    pub fn get_credential(app: &AppHandle, key: &str) -> Result<String, AppError> {
        credential::CredentialManager::get_credential(app, key)
    }

    /// Clears the active session marker without deleting the identity data.
    pub fn clear_active_session(app: &AppHandle) -> Result<(), AppError> {
        let _ = identity::IdentityManager::clear_keyring_entry("active_account");
        let _ = FallbackManager::delete_fallback(app, "active_account");
        Logger::info(app, "[Vault] Active session cleared.", None);
        Ok(())
    }

    /// Performs a full data wipe of all identities, credentials, and settings.
    pub fn clear_all_data(app: &AppHandle) -> Result<(), AppError> {
        Logger::info(app, "[Vault] Initiating full data wipe...", None);

        // Clear all identities
        let identities = Self::list_identities(app);
        for identity in identities {
            let _ = Self::remove_identity(app, &identity.id);
        }

        // Clear active_account and identity_index
        let _ = identity::IdentityManager::clear_keyring_entry("active_account");
        let _ = FallbackManager::delete_fallback(app, "active_account");
        let _ = identity::IdentityManager::clear_keyring_entry("identity_index");
        let _ = FallbackManager::delete_fallback(app, "identity_index");

        // Clear credentials
        let _ = credential::CredentialManager::remove_credential(app, "client_id");
        let _ = credential::CredentialManager::remove_credential(app, "client_secret");
        let _ = credential::CredentialManager::remove_credential(app, "proxy_url");

        Logger::info(app, "[Vault] All application data cleared.", None);
        Ok(())
    }
}
