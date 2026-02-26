// src-tauri/src/tests/fingerprint_test.rs

#[cfg(test)]
mod tests {
    use crate::api::rate_limiter::fingerprint::FingerprintManager;

    #[test]
    fn test_random_user_agent() {
        let ua = FingerprintManager::random_user_agent();
        assert!(!ua.is_empty());
        assert!(ua.contains("Mozilla/5.0"));
    }

    #[test]
    fn test_super_properties_generation() {
        let ua = "TestUA";
        let props = FingerprintManager::generate_super_properties(ua);
        assert!(!props.is_empty());

        // Decode and check
        let decoded =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, props).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&decoded).unwrap();
        assert_eq!(json["browser_user_agent"], ua);
        assert_eq!(json["os"], "Windows");
    }
}
