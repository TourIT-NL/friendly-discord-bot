// src-tauri/src/api/discord/sync.rs

use super::types::{Channel, Guild, Relationship};
use crate::api::rate_limiter::ApiHandle;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::vault::Vault;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn fetch_guilds(app_handle: AppHandle) -> Result<Vec<Guild>, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    Logger::info(
        &app_handle,
        &format!("[SYNC] Fetching guilds (OAuth: {})...", is_bearer),
        None,
    );

    let response_value = api_handle
        .send_request(
            reqwest::Method::GET,
            "https://discord.com/api/v10/users/@me/guilds",
            None,
            &token,
            is_bearer,
        )
        .await?; // Will return serde_json::Value if successful

    Ok(serde_json::from_value(response_value).map_err(AppError::from)?)
}

#[tauri::command]
pub async fn fetch_channels(
    app_handle: AppHandle,
    guild_id: Option<String>,
) -> Result<Vec<Channel>, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();

    if let Some(gid) = guild_id {
        Logger::info(
            &app_handle,
            &format!("[SYNC] Mapping nodes for guild {}", gid),
            None,
        );
                        let response_value = api_handle
                            .send_request(
                                reqwest::Method::GET,
                                &format!("https://discord.com/api/v10/guilds/{}/channels", gid),
                                None,
                                &token,
                                is_bearer,
                            )
                            .await?; // Will return serde_json::Value if successful
        
                        let channels: Vec<Channel> = serde_json::from_value(response_value).map_err(AppError::from)?;        Ok(channels
            .into_iter()
            .filter(|c| c.channel_type == 0 || c.channel_type == 11 || c.channel_type == 12)
            .collect())
    } else {
        Logger::info(&app_handle, "[SYNC] Fetching DM nodes...", None);
        if is_bearer {
            return Err(AppError {
                user_message: "DMs restricted in Official Gate.".into(),
                ..Default::default()
            });
        }
                    let response_value = api_handle
                        .send_request(
                            reqwest::Method::GET,
                            "https://discord.com/api/v10/users/@me/channels",
                            None,
                            &token,
                            is_bearer,
                        )
                        .await?; // Will return serde_json::Value if successful
        
                    let channels: Vec<serde_json::Value> = serde_json::from_value(response_value).map_err(AppError::from)?;        let mut result = Vec::new();
        for ch in channels {
            let ch_type = ch["type"].as_u64().unwrap_or(0);
            if ch_type == 1 || ch_type == 3 {
                                let name = if ch_type == 1 {
                                    ch["recipients"].as_array().and_then(|r| r.first()).and_then(|u| u["username"].as_str()).map(|s| format!("DM with {}", s))
                                } else {
                    ch["name"]
                        .as_str()
                        .map(|s| s.to_string())
                        .or_else(|| Some("Unnamed Group DM".to_string()))
                };
                result.push(Channel {
                    id: ch["id"].as_str().unwrap_or_default().to_string(),
                    name,
                    channel_type: ch_type as u8,
                });
            }
        }
        Ok(result)
    }
}

#[tauri::command]
pub async fn fetch_relationships(app_handle: AppHandle) -> Result<Vec<Relationship>, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    if is_bearer {
        return Err(AppError {
            user_message: "Relationships restricted in Official Gate.".into(),
            ..Default::default()
        });
    }

    Logger::info(&app_handle, "[SYNC] Fetching identity links...", None);
    let response_value = api_handle
        .send_request(
            reqwest::Method::GET,
            "https://discord.com/api/v10/users/@me/relationships",
            None,
            &token,
            is_bearer,
        )
        .await?; // Will return serde_json::Value if successful

    Ok(serde_json::from_value(response_value).map_err(AppError::from)?)
}

#[tauri::command]
pub async fn fetch_preview_messages(
    app_handle: AppHandle,
    channel_id: String,
) -> Result<Vec<serde_json::Value>, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    let response_value = api_handle
        .send_request(
            reqwest::Method::GET,
            &format!(
                "https://discord.com/api/v10/channels/{}/messages?limit=5",
                channel_id
            ),
            None,
            &token,
            is_bearer,
        )
        .await?; // Will return serde_json::Value if successful

    Ok(serde_json::from_value(response_value).map_err(AppError::from)?)}
