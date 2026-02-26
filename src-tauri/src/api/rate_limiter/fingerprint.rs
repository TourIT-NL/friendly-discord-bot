// src-tauri/src/api/rate_limiter/fingerprint.rs

use base64::{Engine as _, engine::general_purpose};
use rand::seq::SliceRandom;
use serde_json::json;
use uuid::Uuid;

/// Represents a browser/OS profile for a genuine Discord request fingerprint.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrowserProfile {
    pub name: String,
    pub user_agent: String,
    pub os: String,
    pub os_version: String,
    pub browser_version: String,
}

#[allow(dead_code)]
pub struct FingerprintManager;

impl FingerprintManager {
    /// A pool of modern browser profiles.
    pub fn get_profiles() -> Vec<BrowserProfile> {
        vec![
            BrowserProfile {
                name: "Chrome (Windows)".to_string(),
                user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36".to_string(),
                os: "Windows".to_string(),
                os_version: "10".to_string(),
                browser_version: "144.0.0.0".to_string(),
            },
            BrowserProfile {
                name: "Firefox (Windows)".to_string(),
                user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/121.0".to_string(),
                os: "Windows".to_string(),
                os_version: "10".to_string(),
                browser_version: "121.0".to_string(),
            },
        ]
    }

    pub fn random_profile() -> BrowserProfile {
        let mut rng = rand::thread_rng();
        Self::get_profiles().choose(&mut rng).unwrap().clone()
    }

    pub fn get_system_locale() -> String {
        "en-US".to_string()
    }

    pub fn generate_accept_language(locale: &str) -> String {
        let lang = locale.split('-').next().unwrap_or("en");
        format!("{},{};q=0.9", locale, lang)
    }

    pub fn generate_synthetic_cookies(locale: &str) -> String {
        let mut rng = rand::thread_rng();
        let dcf = format!("{:x}", Uuid::new_v4()).replace("-", "");
        let sdcf = format!("{:x}", Uuid::new_v4()).replace("-", "");
        format!("__dcfduid={}; __sdcfduid={}; locale={};", dcf, sdcf, locale)
    }

    pub fn generate_super_properties(profile: &BrowserProfile, locale: &str) -> String {
        let json = json!({
            "os": profile.os,
            "browser": profile.name.split(' ').next().unwrap_or("Chrome"),
            "device": "",
            "system_locale": locale,
            "browser_user_agent": profile.user_agent,
            "browser_version": profile.browser_version,
            "os_version": profile.os_version,
            "release_channel": "stable",
            "client_build_number": 501798,
            "client_launch_id": Uuid::new_v4().to_string(),
        });
        general_purpose::STANDARD.encode(json.to_string())
    }

    pub fn generate_client_hints(profile: &BrowserProfile) -> Vec<(&'static str, String)> {
        let platform = format!("\"{}\"", profile.os);
        let major = profile.browser_version.split('.').next().unwrap_or("144");
        vec![
            (
                "sec-ch-ua",
                format!(
                    "\"Not(A:Brand\";v=\"8\", \"Chromium\";v=\"{}\", \"Chrome\";v=\"{}\"",
                    major, major
                ),
            ),
            ("sec-ch-ua-mobile", "?0".to_string()),
            ("sec-ch-ua-platform", platform),
        ]
    }
}
