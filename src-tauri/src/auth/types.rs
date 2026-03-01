// src-tauri/src/auth/types.rs

use serde::{Deserialize, Serialize};

/// Robust OAuth2 configuration for dynamic client management.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub scopes: Vec<String>,
    pub redirect_uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscordStatus {
    pub is_running: bool,
    pub rpc_available: bool,
    pub browser_detected: bool,
    pub active_pid: Option<u32>,
    pub detected_port: Option<u16>,
    pub installation_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub avatar: Option<String>,
    pub email: Option<String>,
    pub mfa_enabled: bool,
    pub flags: u64,
    pub verified: bool,
}

/// Exhaustive mapping of Discord API Error Codes for high-fidelity handling.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum DiscordError {
    General,
    UnknownAccount,
    UnknownApplication,
    UnknownChannel,
    UnknownGuild,
    UnknownIntegration,
    Unauthorized,
    MissingAccess,
    InvalidToken,
    SlowmodeLimit,
    RateLimited,
}

impl DiscordError {
    pub fn from_code(code: u32) -> Self {
        match code {
            10001 => Self::UnknownAccount,
            10002 => Self::UnknownApplication,
            10003 => Self::UnknownChannel,
            10004 => Self::UnknownGuild,
            10005 => Self::UnknownIntegration,
            40001 => Self::Unauthorized,
            50001 => Self::MissingAccess,
            50014 => Self::InvalidToken,
            20016 => Self::SlowmodeLimit,
            429 => Self::RateLimited,
            _ => Self::General,
        }
    }
}
