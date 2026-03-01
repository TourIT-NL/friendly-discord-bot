// src-tauri/src/core/forensics/pii.rs

use crate::core::cache::CacheManager;
use crate::core::error::AppError;
use regex::Regex;
use serde::Serialize;
use std::sync::OnceLock;
use tauri::AppHandle;

static PII_PATTERNS: OnceLock<Vec<(&'static str, Regex)>> = OnceLock::new();

#[derive(Serialize)]
pub struct PIIResult {
    pub message_id: String,
    pub channel_id: String,
    pub detected_types: Vec<String>,
    pub snippet: String,
}

pub struct PIIClassifier;

impl PIIClassifier {
    fn get_patterns() -> &'static Vec<(&'static str, Regex)> {
        PII_PATTERNS.get_or_init(|| {
            vec![
                (
                    "Email",
                    Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
                ),
                (
                    "Credit Card",
                    Regex::new(r"\b(?:\d[ -]*?){13,16}\b").unwrap(),
                ),
                ("IPv4", Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap()),
                ("SSN (US)", Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap()),
                (
                    "Discord Token",
                    Regex::new(r"[a-zA-Z0-9_-]{24}\.[a-zA-Z0-9_-]{6}\.[a-zA-Z0-9_-]{27,38}")
                        .unwrap(),
                ),
                (
                    "BTC Wallet",
                    Regex::new(r"\b[13][a-km-zA-HJ-NP-Z1-9]{25,34}\b").unwrap(),
                ),
                ("ETH Wallet", Regex::new(r"\b0x[a-fA-F0-9]{40}\b").unwrap()),
            ]
        })
    }

    /// Scans the entire local cache for Personally Identifiable Information.
    pub fn scan_cache(app: &AppHandle) -> Result<Vec<PIIResult>, AppError> {
        let messages = CacheManager::search_messages(app, "")?; // Get all cached messages
        let patterns = Self::get_patterns();
        let mut results = Vec::new();

        for msg in messages {
            let content = msg["content"].as_str().unwrap_or_default();
            let mut detected = Vec::new();

            for (name, re) in patterns {
                if re.is_match(content) {
                    detected.push(name.to_string());
                }
            }

            if !detected.is_empty() {
                results.push(PIIResult {
                    message_id: msg["id"].as_str().unwrap_or_default().to_string(),
                    channel_id: msg["channel_id"].as_str().unwrap_or_default().to_string(),
                    detected_types: detected,
                    snippet: content.chars().take(100).collect(), // Redacted snippet would be better, but for now...
                });
            }
        }

        Ok(results)
    }
}
