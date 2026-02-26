// src-tauri/src/auth/types.rs

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

/// Master Application ID for the Privacy Utility (Official Fallback)
pub const MASTER_CLIENT_ID: &str = "1473823776247382097";
/// Master Application Secret for the Privacy Utility (Official Fallback)
pub const MASTER_CLIENT_SECRET: &str = "NzZg8uEiBx6DjxR6EXLUugMR5hiXXf_0";

#[derive(Default)]
pub struct AuthState {
    pub qr_cancel_token: Mutex<Option<CancellationToken>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscordStatus {
    pub is_running: bool,
    pub rpc_available: bool,
    pub browser_detected: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub avatar: Option<String>,
    pub email: Option<String>,
}
