// src-tauri/src/core/logger.rs

use regex::Regex;
use serde::Serialize;
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};
use tracing::{debug, error, info, trace, warn};

static REDACT_REGEXES: OnceLock<Vec<Regex>> = OnceLock::new();

#[derive(Serialize, Clone)]
pub struct LogEvent {
    pub level: &'static str,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
}

pub struct Logger;

impl Logger {
    pub fn redact(message: &str) -> String {
        let regexes = REDACT_REGEXES.get_or_init(|| {
            vec![
                // Discord Token (classic and newer)
                Regex::new(r"[a-zA-Z0-9_-]{24}\.[a-zA-Z0-9_-]{6}\.[a-zA-Z0-9_-]{27,38}").unwrap(),
                // Discord Token (mfa prefixed)
                Regex::new(r"mfa\.[a-zA-Z0-9_-]{84}").unwrap(),
                // Email pattern
                Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap(),
                // IPv4 Addresses
                Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap(),
                // Proxy credentials pattern (user:pass@)
                Regex::new(r"(?i)(socks5|http|https)://[^:]+:[^@]+@").unwrap(),
            ]
        });

        let mut redacted = message.to_string();
        for re in regexes {
            redacted = re.replace_all(&redacted, "[REDACTED]").to_string();
        }
        redacted
    }

    pub fn info(app: &AppHandle, message: &str, metadata: Option<serde_json::Value>) {
        let redacted = Self::redact(message);
        info!("{}", redacted);
        let _ = app.emit(
            "log_event",
            LogEvent {
                level: "info",
                message: redacted,
                metadata,
            },
        );
    }

    pub fn warn(app: &AppHandle, message: &str, metadata: Option<serde_json::Value>) {
        let redacted = Self::redact(message);
        warn!("{}", redacted);
        let _ = app.emit(
            "log_event",
            LogEvent {
                level: "warn",
                message: redacted,
                metadata,
            },
        );
    }

    pub fn error(app: &AppHandle, message: &str, metadata: Option<serde_json::Value>) {
        let redacted = Self::redact(message);
        error!("{}", redacted);
        let _ = app.emit(
            "log_event",
            LogEvent {
                level: "error",
                message: redacted,
                metadata,
            },
        );
    }

    pub fn debug(app: &AppHandle, message: &str, metadata: Option<serde_json::Value>) {
        let redacted = Self::redact(message);
        debug!("{}", redacted);
        let _ = app.emit(
            "log_event",
            LogEvent {
                level: "debug",
                message: redacted,
                metadata,
            },
        );
    }

    pub fn trace(app: &AppHandle, message: &str, metadata: Option<serde_json::Value>) {
        let redacted = Self::redact(message);
        trace!("{}", redacted);
        let _ = app.emit(
            "log_event",
            LogEvent {
                level: "trace",
                message: redacted,
                metadata,
            },
        );
    }
}
