// src-tauri/src/core/vault/encryption.rs

use crate::core::crypto::Crypto;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::vault::state::VaultState;
use keyring::Entry;
use std::fs;
use tauri::{AppHandle, Manager};

/// Manages the application's encryption keys and provides master password integration.
/// Implements a multi-layered security model where the primary vault key is
/// optionally encrypted by a user-provided master password.
pub struct EncryptionManager;

impl EncryptionManager {
    pub const SERVICE_NAME: &'static str = "com.discordprivacy.util";
    pub const ENCRYPTION_KEY_SERVICE_NAME: &'static str = "com.discordprivacy.util.enc_key";
    pub const MASTER_HASH_SERVICE_NAME: &'static str = "com.discordprivacy.util.master_hash";

    /// Retrieves the encryption key from memory, keyring, or fallback file.
    /// If a master password is set but not provided, returns a `vault_locked` error.
    pub fn get_or_create_encryption_key(app: &AppHandle) -> Result<String, AppError> {
        // 1. Check in-memory state first
        if let Some(state) = app.try_state::<VaultState>() {
            let key_guard = state.encryption_key.lock().unwrap();
            if let Some(key) = key_guard.as_ref() {
                return Ok(key.to_string());
            }
        }

        // 2. If no master password is set, we can load from keyring
        if !Self::has_master_password(app)? {
            let key_name = Self::ENCRYPTION_KEY_SERVICE_NAME;
            if let Ok(entry) = Entry::new(Self::SERVICE_NAME, key_name)
                && let Ok(key) = entry.get_password()
            {
                return Ok(key);
            }
            if let Some(path) = super::fallback::FallbackManager::get_fallback_path(app, key_name)
                && path.exists()
                && let Ok(key) = fs::read_to_string(&path)
            {
                return Ok(key);
            }

            // Create new key if none exists
            Logger::warn(
                app,
                "[Vault] No existing encryption key found. Generating fresh identity key.",
                None,
            );
            let new_key = Crypto::generate_key();
            let _ = Entry::new(Self::SERVICE_NAME, key_name).and_then(|e| e.set_password(&new_key));
            if let Some(path) = super::fallback::FallbackManager::get_fallback_path(app, key_name) {
                let _ = fs::write(&path, &new_key);
            }
            return Ok(new_key);
        }

        // 3. If master password IS set but we don't have the key in memory, it's LOCKED
        Err(AppError {
            user_message: "Vault is locked. Master password required.".into(),
            error_code: "vault_locked".into(),
            ..Default::default()
        })
    }

    /// Checks if a master password has been configured for the current OS account.
    pub fn has_master_password(app: &AppHandle) -> Result<bool, AppError> {
        let entry = Entry::new(Self::SERVICE_NAME, Self::MASTER_HASH_SERVICE_NAME)
            .map_err(AppError::from)?;
        if entry.get_password().is_ok() {
            return Ok(true);
        }

        if let Some(path) =
            super::fallback::FallbackManager::get_fallback_path(app, Self::MASTER_HASH_SERVICE_NAME)
            && path.exists()
        {
            return Ok(true);
        }

        Ok(false)
    }

    /// Sets, updates, or removes the master password.
    /// When setting a password, the vault key is re-encrypted using Argon2id-derived keys.
    /// When removing, the vault key is decrypted and stored in plaintext within the secure OS keyring.
    pub fn set_master_password(app: &AppHandle, password: Option<&str>) -> Result<(), AppError> {
        let key_name = Self::ENCRYPTION_KEY_SERVICE_NAME;
        let hash_name = Self::MASTER_HASH_SERVICE_NAME;

        // Get current key (must be unlocked if it was locked)
        let current_key = Self::get_or_create_encryption_key(app)?;

        if let Some(pwd) = password {
            // 1. Hash the password
            let hash = Crypto::hash_password(pwd)?;

            // 2. Derive encryption key for the vault key
            let derived = Crypto::derive_key(pwd, "com.discordprivacy.vault.salt")?;

            // 3. Encrypt the vault key
            let encrypted_vault_key = Crypto::encrypt_raw(&derived, &current_key)?;

            // 4. Store hash and encrypted key
            let _ = Entry::new(Self::SERVICE_NAME, hash_name).and_then(|e| e.set_password(&hash));
            let _ = Entry::new(Self::SERVICE_NAME, key_name)
                .and_then(|e| e.set_password(&encrypted_vault_key));

            if let Some(path) = super::fallback::FallbackManager::get_fallback_path(app, hash_name)
            {
                let _ = fs::write(path, &hash);
            }
            if let Some(path) = super::fallback::FallbackManager::get_fallback_path(app, key_name) {
                let _ = fs::write(path, &encrypted_vault_key);
            }
        } else {
            // Remove master password (decrypt key and store as plaintext)
            if let Ok(entry) = Entry::new(Self::SERVICE_NAME, hash_name) {
                let _ = entry.delete_credential();
            }
            let _ =
                Entry::new(Self::SERVICE_NAME, key_name).and_then(|e| e.set_password(&current_key));

            if let Some(path) = super::fallback::FallbackManager::get_fallback_path(app, hash_name)
            {
                let _ = fs::remove_file(path);
            }
            if let Some(path) = super::fallback::FallbackManager::get_fallback_path(app, key_name) {
                let _ = fs::write(path, &current_key);
            }
        }

        Ok(())
    }

    /// Verifies the master password and returns the decrypted vault key.
    pub fn unlock_with_password(app: &AppHandle, password: &str) -> Result<String, AppError> {
        let hash_name = Self::MASTER_HASH_SERVICE_NAME;
        let key_name = Self::ENCRYPTION_KEY_SERVICE_NAME;

        // 1. Get hash
        let hash = match Entry::new(Self::SERVICE_NAME, hash_name).and_then(|e| e.get_password()) {
            Ok(h) => h,
            Err(_) => {
                if let Some(path) =
                    super::fallback::FallbackManager::get_fallback_path(app, hash_name)
                    && let Ok(h) = fs::read_to_string(path)
                {
                    h
                } else {
                    return Err(AppError {
                        user_message: "No master password set.".into(),
                        ..Default::default()
                    });
                }
            }
        };

        // 2. Verify password
        if !Crypto::verify_password(password, &hash) {
            return Err(AppError {
                user_message: "Incorrect master password.".into(),
                error_code: "invalid_master_password".into(),
                ..Default::default()
            });
        }

        // 3. Get encrypted key
        let encrypted_key =
            match Entry::new(Self::SERVICE_NAME, key_name).and_then(|e| e.get_password()) {
                Ok(k) => k,
                Err(_) => {
                    if let Some(path) =
                        super::fallback::FallbackManager::get_fallback_path(app, key_name)
                        && let Ok(k) = fs::read_to_string(path)
                    {
                        k
                    } else {
                        return Err(AppError {
                            user_message: "Vault key missing.".into(),
                            ..Default::default()
                        });
                    }
                }
            };

        // 4. Decrypt key
        let derived = Crypto::derive_key(password, "com.discordprivacy.vault.salt")?;
        Crypto::decrypt_raw(&derived, &encrypted_key)
    }
}
