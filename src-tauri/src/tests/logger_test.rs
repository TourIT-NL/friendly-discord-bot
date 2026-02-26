// src-tauri/src/tests/logger_test.rs

#[cfg(test)]
mod tests {
    use crate::core::logger::Logger;

    #[test]
    fn test_token_redaction() {
        // Use a string that matches the Discord token regex pattern but is obviously mock
        let token = "AAAAAAAAAAAAAAAAAAAAAAAA.BBBBBB.CCCCCCCCCCCCCCCCCCCCCCCCCCC";
        let message = format!("User token is: {}", token);
        let redacted = Logger::redact(&message);
        assert!(!redacted.contains(token));
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn test_email_redaction() {
        let email = "user@example.com";
        let message = format!("Contact us at {}", email);
        let redacted = Logger::redact(&message);
        assert!(!redacted.contains(email));
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn test_proxy_redaction() {
        let proxy = "socks5://user:password@127.0.0.1:9050";
        let redacted = Logger::redact(proxy);
        assert!(!redacted.contains("password"));
        assert!(redacted.contains("[REDACTED]"));
    }
}
