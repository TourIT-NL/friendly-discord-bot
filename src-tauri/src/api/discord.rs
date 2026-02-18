// src-tauri/src/api/discord.rs

use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Manager, Emitter};
use crate::api::rate_limiter::ApiHandle;
use crate::core::error::AppError;
use keyring::Entry;
use tracing::{info, warn, error};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Guild {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub owner: bool,
    pub permissions: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Channel {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub channel_type: u8, // 0 for text, 2 for voice, etc.
}

#[tauri::command]
pub async fn fetch_guilds(app_handle: AppHandle) -> Result<Vec<Guild>, AppError> {
    info!("Fetching user guilds...");

    // 1. Get tokens from keyring
    let entry = Entry::new("discord_privacy_util", "discord_user")?;
    let password = entry.get_password()?;
    
    let access_token = password.lines()
        .find(|line| line.starts_with("ACCESS_TOKEN="))
        .and_then(|line| line.strip_prefix("ACCESS_TOKEN="))
        .ok_or_else(|| AppError {
            user_message: "Access token not found in secure store. Please login again.".to_string(),
            error_code: "access_token_missing".to_string(),
            technical_details: None,
        })?;

    // 2. Use ApiHandle to make the request
    let api_handle = app_handle.state::<ApiHandle>();
    let response = api_handle.send_request(
        reqwest::Method::GET,
        "https://discord.com/api/users/@me/guilds",
        None,
        access_token
    ).await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!("Failed to fetch guilds: {} - {}", status, body);
        return Err(AppError {
            user_message: "Failed to fetch guilds from Discord.".to_string(),
            error_code: "guilds_fetch_failure".to_string(),
            technical_details: Some(format!("Status: {}, Body: {}", status, body)),
        });
    }

    let guilds: Vec<Guild> = response.json().await?;
    info!("Successfully fetched {} guilds.", guilds.len());

    Ok(guilds)
}

#[tauri::command]
pub async fn fetch_channels(app_handle: AppHandle, guild_id: String) -> Result<Vec<Channel>, AppError> {
    info!("Fetching channels for guild ID {}...", guild_id);

    // 1. Get tokens from keyring
    let entry = Entry::new("discord_privacy_util", "discord_user")?;
    let password = entry.get_password()?;
    
    let access_token = password.lines()
        .find(|line| line.starts_with("ACCESS_TOKEN="))
        .and_then(|line| line.strip_prefix("ACCESS_TOKEN="))
        .ok_or_else(|| AppError {
            user_message: "Access token not found in secure store. Please login again.".to_string(),
            error_code: "access_token_missing".to_string(),
            technical_details: None,
        })?;

    // 2. Use ApiHandle to make the request
    let api_handle = app_handle.state::<ApiHandle>();
    let response = api_handle.send_request(
        reqwest::Method::GET,
        &format!("https://discord.com/api/guilds/{}/channels", guild_id),
        None,
        access_token
    ).await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!("Failed to fetch channels for guild {}: {} - {}", guild_id, status, body);
        return Err(AppError {
            user_message: format!("Failed to fetch channels for guild {}.", guild_id),
            error_code: "channels_fetch_failure".to_string(),
            technical_details: Some(format!("Status: {}, Body: {}", status, body)),
        });
    }

    let channels: Vec<Channel> = response.json().await?;
    let text_channels: Vec<Channel> = channels.into_iter()
        .filter(|c| c.channel_type == 0 || c.channel_type == 11 || c.channel_type == 12)
        .collect();

    info!("Successfully fetched {} text/thread channels.", text_channels.len());

    Ok(text_channels)
}

#[tauri::command]
pub async fn bulk_delete_messages(
    app_handle: AppHandle,
    window: tauri::Window,
    channel_ids: Vec<String>,
    start_time: Option<u64>,
    end_time: Option<u64>,
) -> Result<(), AppError> {
    info!("Starting bulk message deletion for {} channels...", channel_ids.len());

    let entry = Entry::new("discord_privacy_util", "discord_user")?;
    let password = entry.get_password()?;
    let access_token = password.lines()
        .find(|line| line.starts_with("ACCESS_TOKEN="))
        .and_then(|line| line.strip_prefix("ACCESS_TOKEN="))
        .ok_or_else(|| AppError {
            user_message: "Access token not found in secure store. Please login again.".to_string(),
            error_code: "access_token_missing".to_string(),
            technical_details: None,
        })?;

    let api_handle = app_handle.state::<ApiHandle>();

    for (index, channel_id) in channel_ids.iter().enumerate() {
        info!("Processing channel {}/{} (ID: {})", index + 1, channel_ids.len(), channel_id);
        
        let _ = window.emit("deletion_progress", serde_json::json!({
            "current_channel": index + 1,
            "total_channels": channel_ids.len(),
            "channel_id": channel_id,
            "deleted_count": 0,
            "status": "fetching"
        }));

        let mut deleted_in_channel = 0;
        let mut last_message_id: Option<String> = None;

        loop {
            let mut url = format!("https://discord.com/api/channels/{}/messages?limit=100", channel_id);
            if let Some(before_id) = &last_message_id {
                url.push_str(&format!("&before={}", before_id));
            }

            let response = api_handle.send_request(
                reqwest::Method::GET,
                &url,
                None,
                access_token
            ).await?;

            if !response.status().is_success() {
                error!("Failed to fetch messages for channel {}: {}", channel_id, response.status());
                break;
            }

            let messages: Vec<serde_json::Value> = response.json().await?;
            if messages.is_empty() {
                break;
            }

            last_message_id = messages.last().and_then(|m| m["id"].as_str()).map(|s| s.to_string());

            for msg in messages {
                let msg_id = msg["id"].as_str().unwrap_or_default();
                let timestamp_str = msg["timestamp"].as_str().unwrap_or_default();
                
                let timestamp = chrono::DateTime::parse_from_rfc3339(timestamp_str)
                    .map(|dt| dt.timestamp_millis() as u64)
                    .unwrap_or(0);

                let in_range = match (start_time, end_time) {
                    (Some(s), Some(e)) => timestamp >= s && timestamp <= e,
                    (Some(s), None) => timestamp >= s,
                    (None, Some(e)) => timestamp <= e,
                    (None, None) => true,
                };

                if in_range {
                    let del_url = format!("https://discord.com/api/channels/{}/messages/{}", channel_id, msg_id);
                    let del_response = api_handle.send_request(
                        reqwest::Method::DELETE,
                        &del_url,
                        None,
                        access_token
                    ).await?;

                    if del_response.status().is_success() || del_response.status() == 404 {
                        deleted_in_channel += 1;
                        if deleted_in_channel % 10 == 0 {
                            let _ = window.emit("deletion_progress", serde_json::json!({
                                "current_channel": index + 1,
                                "total_channels": channel_ids.len(),
                                "channel_id": channel_id,
                                "deleted_count": deleted_in_channel,
                                "status": "deleting"
                            }));
                        }
                    } else if del_response.status() == 403 {
                        warn!("Permission denied to delete message {} in channel {}", msg_id, channel_id);
                    } else {
                        error!("Failed to delete message {}: {}", msg_id, del_response.status());
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        info!("Finished channel {}. Deleted {} messages.", channel_id, deleted_in_channel);
    }

    info!("Bulk message deletion complete.");
    let _ = window.emit("deletion_complete", ());
    Ok(())
}
