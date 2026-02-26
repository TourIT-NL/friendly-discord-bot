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

    let json = api_handle
        .send_request_json(
            reqwest::Method::GET,
            "https://discord.com/api/v9/users/@me/guilds",
            None,
            &token,
            is_bearer,
            None,
        )
        .await?;

    serde_json::from_value(json).map_err(AppError::from)
}

#[tauri::command]
pub async fn fetch_channels(
    app_handle: AppHandle,
    guild_id: Option<String>,
) -> Result<Vec<Channel>, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();

    if let Some(gid) = guild_id {
        let json = api_handle
            .send_request_json(
                reqwest::Method::GET,
                &format!("https://discord.com/api/v9/guilds/{}/channels", gid),
                None,
                &token,
                is_bearer,
                None,
            )
            .await?;

        let channels: Vec<Channel> = serde_json::from_value(json).map_err(AppError::from)?;
        Ok(channels
            .into_iter()
            .filter(|c| [0, 5, 11, 12, 15].contains(&c.channel_type))
            .collect())
    } else {
        if is_bearer {
            return Err(AppError::new("DMs restricted", "dm_restricted"));
        }
        let json = api_handle
            .send_request_json(
                reqwest::Method::GET,
                "https://discord.com/api/v9/users/@me/channels",
                None,
                &token,
                is_bearer,
                None,
            )
            .await?;

        let channels: Vec<serde_json::Value> =
            serde_json::from_value(json).map_err(AppError::from)?;
        let mut result = Vec::new();
        for ch in channels {
            let ch_type = ch["type"].as_u64().unwrap_or(0);
            if ch_type == 1 || ch_type == 3 {
                let name = if ch_type == 1 {
                    ch["recipients"]
                        .as_array()
                        .and_then(|r| r.first())
                        .and_then(|u| u["username"].as_str())
                        .map(|s| format!("DM with {}", s))
                } else {
                    ch["name"]
                        .as_str()
                        .map(|s| s.to_string())
                        .or(Some("Unnamed Group DM".to_string()))
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

    let json = api_handle
        .send_request_json(
            reqwest::Method::GET,
            "https://discord.com/api/v9/users/@me/relationships",
            None,
            &token,
            is_bearer,
            None,
        )
        .await?;

    serde_json::from_value(json).map_err(AppError::from)
}

#[tauri::command]
pub async fn fetch_preview_messages(
    app_handle: AppHandle,
    channel_id: String,
) -> Result<Vec<serde_json::Value>, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();

    let json_res = api_handle
        .send_request_json(
            reqwest::Method::GET,
            &format!(
                "https://discord.com/api/v9/channels/{}/messages?limit=5",
                channel_id
            ),
            None,
            &token,
            is_bearer,
            None,
        )
        .await;

    match json_res {
        Ok(json) => serde_json::from_value(json).map_err(AppError::from),
        Err(e) => {
            Logger::warn(
                &app_handle,
                &format!(
                    "[SYNC] Failed to fetch preview for {}: {}",
                    channel_id, e.user_message
                ),
                None,
            );
            Ok(vec![])
        }
    }
}
