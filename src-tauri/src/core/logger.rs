// src-tauri/src/core/logger.rs

use tauri::{AppHandle, Emitter};
use serde::Serialize;
use tracing::{debug, error, info, trace, warn};

#[derive(Serialize, Clone)]
pub struct LogEvent {
    pub level: &'static str,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
}

pub struct Logger;

impl Logger {
    pub fn info(app: &AppHandle, message: &str, metadata: Option<serde_json::Value>) {
        info!("{}", message);
        let _ = app.emit("log_event", LogEvent {
            level: "info",
            message: message.to_string(),
            metadata,
        });
    }

    pub fn warn(app: &AppHandle, message: &str, metadata: Option<serde_json::Value>) {
        warn!("{}", message);
        let _ = app.emit("log_event", LogEvent {
            level: "warn",
            message: message.to_string(),
            metadata,
        });
    }

    pub fn error(app: &AppHandle, message: &str, metadata: Option<serde_json::Value>) {
        error!("{}", message);
        let _ = app.emit("log_event", LogEvent {
            level: "error",
            message: message.to_string(),
            metadata,
        });
    }

    pub fn debug(app: &AppHandle, message: &str, metadata: Option<serde_json::Value>) {
        debug!("{}", message);
        let _ = app.emit("log_event", LogEvent {
            level: "debug",
            message: message.to_string(),
            metadata,
        });
    }

    pub fn trace(app: &AppHandle, message: &str, metadata: Option<serde_json::Value>) {
        trace!("{}", message);
        let _ = app.emit("log_event", LogEvent {
            level: "trace",
            message: message.to_string(),
            metadata,
        });
    }
}
