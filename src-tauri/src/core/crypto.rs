// src-tauri/src/core/crypto.rs

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use base64::{Engine as _, engine::general_purpose};
use rand_core::RngCore;
use zeroize::{Zeroize, Zeroizing};

use crate::core::error::AppError;

const NONCE_LEN: usize = 12; // AES-GCM standard nonce size

/// Forensic-grade cryptographic utility for secure data handling.
/// Provides AES-256-GCM encryption, Argon2id key derivation, and memory hardening via Zeroize.
pub struct Crypto;

impl Crypto {
    /// Generates a new cryptographically secure random 256-bit AES key.
    /// Returns the key as a base64-encoded string.
    /// Memory occupied by the raw key bytes is zeroized after encoding.
    pub fn generate_key() -> String {
        let mut key_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);
        let encoded = general_purpose::STANDARD.encode(key_bytes);
        key_bytes.zeroize();
        encoded
    }

    /// Derives a 256-bit encryption key from a high-entropy password and salt.
    /// Uses the Argon2id algorithm with default parameters (m=65536, t=3, p=4).
    /// Returns a `Zeroizing` wrapper around the derived key to ensure memory safety.
    pub fn derive_key(password: &str, salt: &str) -> Result<Zeroizing<[u8; 32]>, AppError> {
        let mut key = [0u8; 32];
        let salt_bytes = salt.as_bytes();

        Argon2::default()
            .hash_password_into(password.as_bytes(), salt_bytes, &mut key)
            .map_err(|e| AppError {
                user_message: "Key derivation failed.".into(),
                error_code: "crypto_kdf_failed".into(),
                technical_details: Some(e.to_string()),
            })?;

        Ok(Zeroizing::new(key))
    }

    /// Verifies a plaintext password against a stored Argon2id hash.
    /// This is a timing-safe comparison to prevent side-channel attacks.
    pub fn verify_password(password: &str, hash: &str) -> bool {
        match PasswordHash::new(hash) {
            Ok(parsed_hash) => Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok(),
            Err(_) => false,
        }
    }

    /// Hashes a password for persistent storage using the Argon2id algorithm.
    /// Generates a unique random salt for each hash operation.
    pub fn hash_password(password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| AppError {
                user_message: "Password hashing failed.".into(),
                error_code: "crypto_hash_failed".into(),
                technical_details: Some(e.to_string()),
            })
    }

    /// Encrypts plaintext using AES-256-GCM with a base64-encoded key.
    /// Appends the 12-byte random nonce to the ciphertext before base64 encoding.
    /// Returns a base64 string of [nonce][ciphertext].
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

    /// Encrypts plaintext using AES-256-GCM with a raw 32-byte key.
    /// Appends the 12-byte random nonce to the ciphertext before base64 encoding.
    pub fn encrypt_raw(key: &[u8; 32], plaintext: &str) -> Result<String, AppError> {
        let cipher = Aes256Gcm::new(key.into());

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

    /// Decrypts a base64 string produced by `encrypt`.
    /// Extracts the nonce from the first 12 bytes of the decoded data.
    /// Zeroizes sensitive intermediate buffers.
    pub fn decrypt(key_base64: &str, ciphertext_base64: &str) -> Result<String, AppError> {
        let mut key_bytes = general_purpose::STANDARD.decode(key_base64)?;
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        let decoded = general_purpose::STANDARD.decode(ciphertext_base64)?;

        if decoded.len() < NONCE_LEN {
            key_bytes.zeroize();
            return Err(AppError {
                user_message: "Decryption failed: invalid data length.".into(),
                error_code: "crypto_decrypt_invalid_len".into(),
                technical_details: None,
            });
        }

        let nonce = Nonce::from_slice(&decoded[..NONCE_LEN]);
        let ciphertext = &decoded[NONCE_LEN..];

        let plaintext_bytes = cipher.decrypt(nonce, ciphertext).map_err(|e| {
            key_bytes.zeroize();
            AppError {
                user_message: "Decryption failed.".into(),
                error_code: "crypto_decrypt_failed".into(),
                technical_details: Some(e.to_string()),
            }
        })?;

        key_bytes.zeroize();
        Ok(String::from_utf8(plaintext_bytes)?)
    }

    /// Decrypts a base64 string produced by `encrypt_raw` using a raw 32-byte key.
    pub fn decrypt_raw(key: &[u8; 32], ciphertext_base64: &str) -> Result<String, AppError> {
        let cipher = Aes256Gcm::new(key.into());

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
