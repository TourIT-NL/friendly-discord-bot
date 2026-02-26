// src-tauri/src/tests/crypto_test.rs

#[cfg(test)]
mod tests {
    use crate::core::crypto::Crypto;

    #[test]
    fn test_key_generation() {
        let key1 = Crypto::generate_key();
        let key2 = Crypto::generate_key();
        assert_ne!(key1, key2);
        assert_eq!(key1.len(), 44); // base64 of 32 bytes
    }

    #[test]
    fn test_encryption_decryption() {
        let key = Crypto::generate_key();
        let plaintext = "Sensitive Discord Token";

        let encrypted = Crypto::encrypt(&key, plaintext).expect("Encryption failed");
        assert_ne!(plaintext, encrypted);

        let decrypted = Crypto::decrypt(&key, &encrypted).expect("Decryption failed");
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_password_hashing_verification() {
        let password = "MySecretPassword123!";
        let hash = Crypto::hash_password(password).expect("Hashing failed");

        assert!(Crypto::verify_password(password, &hash));
        assert!(!Crypto::verify_password("WrongPassword", &hash));
    }

    #[test]
    fn test_key_derivation() {
        let password = "VaultPassword";
        let salt = "fixed_salt_for_test";

        let key1 = Crypto::derive_key(password, salt).expect("KDF failed");
        let key2 = Crypto::derive_key(password, salt).expect("KDF failed");

        assert_eq!(*key1, *key2);

        let key3 = Crypto::derive_key("DifferentPassword", salt).expect("KDF failed");
        assert_ne!(*key1, *key3);
    }
}
