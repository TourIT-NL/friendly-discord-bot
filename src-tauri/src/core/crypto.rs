// src-tauri/src/core/crypto.rs

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use base64::{Engine as _, engine::general_purpose};
use rand_core::RngCore;

use crate::core::error::AppError;

const NONCE_LEN: usize = 12; // AES-GCM standard nonce size

pub struct Crypto;

impl Crypto {
    /// Generates a new random 256-bit AES key.
    pub fn generate_key() -> String {
        let mut key_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);
        general_purpose::STANDARD.encode(key_bytes)
    }

    /// Encrypts data using AES-256 GCM.
    /// Returns a base64-encoded string of the nonce + ciphertext.
    pub fn encrypt(key_base64: &str, plaintext: &str) -> Result<String, AppError> {
        let key_bytes = general_purpose::STANDARD.decode(key_base64)?;
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| AppError {
                user_message: "Encryption failed.".into(),
                error_code: "crypto_encrypt_failed".into(),
                technical_details: Some(e.to_string()),
            })?;

        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(general_purpose::STANDARD.encode(result))
    }

    /// Decrypts data using AES-256 GCM.
    /// Expects a base64-encoded string of the nonce + ciphertext.
    pub fn decrypt(key_base64: &str, ciphertext_base64: &str) -> Result<String, AppError> {
        let key_bytes = general_purpose::STANDARD.decode(key_base64)?;
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        let decoded = general_purpose::STANDARD.decode(ciphertext_base64)?;

        if decoded.len() < NONCE_LEN {
            return Err(AppError {
                user_message: "Decryption failed: invalid data length.".into(),
                error_code: "crypto_decrypt_invalid_len".into(),
                technical_details: None,
            });
        }

        let nonce = Nonce::from_slice(&decoded[..NONCE_LEN]);
        let ciphertext = &decoded[NONCE_LEN..];

        let plaintext_bytes = cipher.decrypt(nonce, ciphertext).map_err(|e| AppError {
            user_message: "Decryption failed.".into(),
            error_code: "crypto_decrypt_failed".into(),
            technical_details: Some(e.to_string()),
        })?;

        Ok(String::from_utf8(plaintext_bytes)?)
    }
}
