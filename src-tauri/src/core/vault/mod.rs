// src-tauri/src/core/vault/mod.rs

pub mod credential;
pub mod encryption;
pub mod fallback;
pub mod identity;

pub use identity::DiscordIdentity;

use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::vault::fallback::FallbackManager; // Added import
use tauri::AppHandle;

pub struct Vault;

impl Vault {
    pub fn save_identity(app: &AppHandle, identity: DiscordIdentity) -> Result<(), AppError> {
        identity::IdentityManager::save_identity(app, identity)
    }

    pub fn get_active_identity(app: &AppHandle) -> Result<DiscordIdentity, AppError> {
        identity::IdentityManager::get_active_identity(app)
    }

    pub fn get_active_token(app: &AppHandle) -> Result<(String, bool), AppError> {
        identity::IdentityManager::get_active_token(app)
    }

    #[allow(dead_code)]
    pub fn get_identity(app: &AppHandle, id: &str) -> Result<DiscordIdentity, AppError> {
        identity::IdentityManager::get_identity(app, id)
    }

    pub fn list_identities(app: &AppHandle) -> Vec<DiscordIdentity> {
        identity::IdentityManager::list_identities(app)
    }

    pub fn remove_identity(app: &AppHandle, id: &str) -> Result<(), AppError> {
        identity::IdentityManager::remove_identity(app, id)
    }

    pub fn set_credential(app: &AppHandle, key: &str, value: &str) -> Result<(), AppError> {
        credential::CredentialManager::set_credential(app, key, value)
    }

    pub fn get_credential(app: &AppHandle, key: &str) -> Result<String, AppError> {
        credential::CredentialManager::get_credential(app, key)
    }

    pub fn clear_active_session(app: &AppHandle) -> Result<(), AppError> {
        let _ = identity::IdentityManager::clear_keyring_entry("active_account");
        let _ = FallbackManager::delete_fallback(app, "active_account");
        Logger::info(app, "[Vault] Active session cleared.", None);
        Ok(())
    }

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

        Logger::info(app, "[Vault] All application data cleared.", None);
        Ok(())
    }
}
