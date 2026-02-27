// src-tauri/src/auth/state.rs

use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

#[derive(Default)]
pub struct AuthState {
    pub qr_cancel_token: Mutex<Option<CancellationToken>>,
}
