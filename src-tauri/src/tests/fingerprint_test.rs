// src-tauri/src/tests/fingerprint_test.rs

#[cfg(test)]
mod tests {
    use crate::api::rate_limiter::fingerprint::FingerprintManager;

    #[test]
    fn test_random_profile() {
        let profile = FingerprintManager::random_profile();
        assert!(!profile.user_agent.is_empty());
        assert!(profile.user_agent.contains("Mozilla/5.0"));
    }

    #[test]
    fn test_super_properties_generation() {
        let profile = FingerprintManager::random_profile();
        let locale = "en-US";
        let props = FingerprintManager::generate_super_properties(&profile, locale);
        assert!(!props.is_empty());

        // Decode and check
        let decoded =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, props).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&decoded).unwrap();
        assert_eq!(json["browser_user_agent"], profile.user_agent);
        assert_eq!(json["system_locale"], locale);
    }

    #[test]
    fn test_accept_language_generation() {
        let locale = "en-US";
        let header = FingerprintManager::generate_accept_language(locale);
        assert!(header.contains("en-US"));
        assert!(header.contains("q=0.9"));
    }

    #[test]
    fn test_synthetic_cookies() {
        let locale = "en-US";
        let cookies = FingerprintManager::generate_synthetic_cookies(locale);
        assert!(cookies.contains("__dcfduid="));
        assert!(cookies.contains("locale=en-US"));
    }
}
