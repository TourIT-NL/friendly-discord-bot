// src-tauri/src/core/vault/encryption.rs

use crate::core::crypto::Crypto;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use keyring::Entry;
use std::fs;
use std::sync::{Mutex, OnceLock};
use tauri::AppHandle;

pub static KEY_CACHE: OnceLock<Mutex<Option<String>>> = OnceLock::new();

pub struct EncryptionManager;

impl EncryptionManager {
    pub const SERVICE_NAME: &'static str = "com.discordprivacy.util";
    pub const ENCRYPTION_KEY_SERVICE_NAME: &'static str = "com.discordprivacy.util.enc_key";

    pub fn get_or_create_encryption_key(app: &AppHandle) -> Result<String, AppError> {
        if let Some(key_mutex) = KEY_CACHE.get() {
            let key_guard = key_mutex.lock().unwrap();
            if let Some(cached_key) = key_guard.as_ref() {
                return Ok(cached_key.clone());
            }
        }

        let key_name = Self::ENCRYPTION_KEY_SERVICE_NAME;
        
        for attempt in 1..=3 {
            if let Ok(entry) = Entry::new(Self::SERVICE_NAME, key_name)
                && let Ok(key) = entry.get_password() {
                    let mutex = KEY_CACHE.get_or_init(|| Mutex::new(None));
                    mutex.lock().unwrap().replace(key.clone());
                    return Ok(key);
            }
            if let Some(path) = super::fallback::FallbackManager::get_fallback_path(app, key_name)
                && path.exists()
                && let Ok(key) = fs::read_to_string(&path) {
                    let mutex = KEY_CACHE.get_or_init(|| Mutex::new(None));
                    mutex.lock().unwrap().replace(key.clone());
                    return Ok(key);
            }
            std::thread::sleep(std::time::Duration::from_millis(100 * attempt));
        }

        Logger::warn(app, "[Vault] No existing encryption key found. Generating fresh identity key.", None);
        let new_key = Crypto::generate_key();
        
        if let Ok(entry) = Entry::new(Self::SERVICE_NAME, key_name) {
            let _ = entry.set_password(&new_key);
        }
        
        if let Some(path) = super::fallback::FallbackManager::get_fallback_path(app, key_name) {
            let _ = fs::write(&path, &new_key);
        }

        let mutex = KEY_CACHE.get_or_init(|| Mutex::new(None));
        mutex.lock().unwrap().replace(new_key.clone());
        Ok(new_key)
    }
}
