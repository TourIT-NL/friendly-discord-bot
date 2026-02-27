// src-tauri/src/api/discord/verification.rs

use crate::api::rate_limiter::ApiHandle;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::vault::Vault;
use tauri::{AppHandle, Manager};

#[derive(serde::Serialize)]
pub struct VerificationReport {
    pub confirmed_deleted: usize,
    pub cached_ghosts: Vec<String>, // IDs that still returned data
}

#[tauri::command]
pub async fn verify_erasure(
    app_handle: AppHandle,
    message_ids: Vec<String>,
    channel_id: String,
) -> Result<VerificationReport, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();

    Logger::info(
        &app_handle,
        &format!(
            "[VERIFY] Starting verification pass for {} targets",
            message_ids.len()
        ),
        None,
    );

    let mut confirmed = 0;
    let mut ghosts = Vec::new();

    for id in message_ids {
        let url = format!(
            "https://discord.com/api/v9/channels/{}/messages/{}",
            channel_id, id
        );
        let res = api_handle
            .send_request_json(reqwest::Method::GET, &url, None, &token, is_bearer, None)
            .await;

        match res {
            Err(e) if e.error_code.contains("404") => confirmed += 1,
            Ok(_) => ghosts.push(id),
            _ => {}
        }
    }

    Ok(VerificationReport {
        confirmed_deleted: confirmed,
        cached_ghosts: ghosts,
    })
}
