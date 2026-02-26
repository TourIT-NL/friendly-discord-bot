// src-tauri/src/api/rate_limiter/fingerprint.rs

use base64::{Engine as _, engine::general_purpose};
use rand::seq::SliceRandom;

/// Generates realistic browser fingerprints to blend application traffic
/// with regular browser-based Discord usage.
#[allow(dead_code)]
pub struct FingerprintManager;

impl FingerprintManager {
    /// A pool of modern browser User-Agents.
    const USER_AGENTS: &[&'static str] = &[
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36",
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/121.0",
    ];

    /// Returns a random User-Agent from the pool.
    pub fn random_user_agent() -> &'static str {
        let mut rng = rand::thread_rng();
        Self::USER_AGENTS
            .choose(&mut rng)
            .unwrap_or(&Self::USER_AGENTS[0])
    }

    /// Generates a base64-encoded `x-super-properties` header matching the User-Agent.
    /// This header is used by Discord to track client metadata.
    pub fn generate_super_properties(user_agent: &str) -> String {
        let json = serde_json::json!({
            "os": "Windows",
            "browser": "Chrome",
            "device": "",
            "system_locale": "en-US",
            "has_client_mods": false,
            "browser_user_agent": user_agent,
            "browser_version": "144.0.0.0",
            "os_version": "10",
            "referrer": "",
            "referring_domain": "",
            "referrer_current": "https://discord.com/",
            "referring_domain_current": "discord.com",
            "release_channel": "stable",
            "client_build_number": 500334,
            "client_event_source": null
        });
        general_purpose::STANDARD.encode(json.to_string())
    }
}
