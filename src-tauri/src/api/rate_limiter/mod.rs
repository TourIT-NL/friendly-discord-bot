// src-tauri/src/api/rate_limiter/mod.rs

pub mod actor;
pub mod fingerprint;
pub mod handle;
pub mod types;

pub use actor::RateLimiterActor;
pub use handle::ApiHandle;

use crate::core::error::AppError;
use crate::core::vault::Vault;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn set_proxy(app_handle: AppHandle, proxy_url: Option<String>) -> Result<(), AppError> {
    Vault::set_credential(&app_handle, "proxy_url", proxy_url.as_deref().unwrap_or(""))?;
    let api_handle = app_handle.state::<ApiHandle>();
    api_handle.rebuild_client().await?;
    Ok(())
}
