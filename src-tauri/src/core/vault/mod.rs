// src-tauri/src/core/vault/mod.rs

pub mod credential;
pub mod encryption;
pub mod fallback;
pub mod identity;

pub use identity::DiscordIdentity;

use crate::core::error::AppError;
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
}
