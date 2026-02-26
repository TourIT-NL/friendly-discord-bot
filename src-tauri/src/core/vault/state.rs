// src-tauri/src/core/vault/state.rs

use std::sync::Mutex;
use zeroize::Zeroizing;

pub struct VaultState {
    pub encryption_key: Mutex<Option<Zeroizing<String>>>,
}

impl Default for VaultState {
    fn default() -> Self {
        Self {
            encryption_key: Mutex::new(None),
        }
    }
}
