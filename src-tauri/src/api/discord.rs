// src-tauri/src/api/discord.rs

use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Manager, Emitter};
use crate::api::rate_limiter::ApiHandle;
use crate::core::error::AppError;
use keyring::Entry;

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
    pub channel_type: u8,
}

const KEYRING_SERVICE: &str = "discord_privacy_util_v3";

fn get_stored_token() -> Result<(String, bool), AppError> {
    let entry = Entry::new(KEYRING_SERVICE, "discord_user")?;
    let password = entry.get_password()?;
    
    let token = password.lines()
        .find(|line| line.starts_with("TOKEN="))
        .and_then(|line| line.strip_prefix("TOKEN="))
        .ok_or_else(|| AppError { user_message: "Token not found. Please login.".into(), ..Default::default() })?;

    let is_bearer = password.lines()
        .find(|line| line.starts_with("TYPE="))
        .map(|line| line.contains("oauth"))
        .unwrap_or(false);

    Ok((token.to_string(), is_bearer))
}

#[tauri::command]
pub async fn fetch_guilds(app_handle: AppHandle) -> Result<Vec<Guild>, AppError> {
    let (token, is_bearer) = get_stored_token()?;
    let api_handle = app_handle.state::<ApiHandle>();
    let response = api_handle.send_request(reqwest::Method::GET, "https://discord.com/api/users/@me/guilds", None, &token, is_bearer).await?;
    if !response.status().is_success() { return Err(AppError { user_message: "Failed to fetch guilds.".into(), ..Default::default() }); }
    Ok(response.json().await?)
}

#[tauri::command]
pub async fn fetch_channels(app_handle: AppHandle, guild_id: String) -> Result<Vec<Channel>, AppError> {
    let (token, is_bearer) = get_stored_token()?;
    let api_handle = app_handle.state::<ApiHandle>();
    let response = api_handle.send_request(reqwest::Method::GET, &format!("https://discord.com/api/guilds/{}/channels", guild_id), None, &token, is_bearer).await?;
    if !response.status().is_success() { return Err(AppError { user_message: "Failed to fetch channels.".into(), ..Default::default() }); }
    let channels: Vec<Channel> = response.json().await?;
    Ok(channels.into_iter().filter(|c| c.channel_type == 0 || c.channel_type == 11 || c.channel_type == 12).collect())
}

#[tauri::command]
pub async fn bulk_delete_messages(
    app_handle: AppHandle,
    window: tauri::Window,
    channel_ids: Vec<String>,
    start_time: Option<u64>,
    end_time: Option<u64>,
) -> Result<(), AppError> {
    let (token, is_bearer) = get_stored_token()?;
    let api_handle = app_handle.state::<ApiHandle>();
    let mut deleted_total = 0;

    for (i, channel_id) in channel_ids.iter().enumerate() {
        let mut last_message_id: Option<String> = None;
        'message_loop: loop {
            let mut url = format!("https://discord.com/api/channels/{}/messages?limit=100", channel_id);
            if let Some(before) = &last_message_id { url.push_str(&format!("&before={}", before)); }

            let response = api_handle.send_request(reqwest::Method::GET, &url, None, &token, is_bearer).await?;
            if !response.status().is_success() { break; }

            let messages: Vec<serde_json::Value> = response.json().await?;
            if messages.is_empty() { break; }

            last_message_id = messages.last().and_then(|m| m["id"].as_str().map(|s| s.to_string()));

            for msg in messages {
                let timestamp = chrono::DateTime::parse_from_rfc3339(msg["timestamp"].as_str().unwrap_or_default()).map(|dt| dt.timestamp_millis() as u64).unwrap_or(0);
                if let (Some(start), Some(end)) = (start_time, end_time) { if !(timestamp >= start && timestamp <= end) { continue; }}
                if let (Some(start), None) = (start_time, end_time) { if timestamp < start { if last_message_id.is_some() { break 'message_loop; } else { continue; } }}
                if let (None, Some(end)) = (start_time, end_time) { if timestamp > end { continue; }}

                let del_url = format!("https://discord.com/api/channels/{}/messages/{}", channel_id, msg["id"].as_str().unwrap_or_default());
                let _ = api_handle.send_request(reqwest::Method::DELETE, &del_url, None, &token, is_bearer).await;
                deleted_total += 1;

                if deleted_total % 10 == 0 {
                    let _ = window.emit("deletion_progress", serde_json::json!({
                        "current_channel": i + 1,
                        "total_channels": channel_ids.len(),
                        "channel_id": channel_id,
                        "deleted_count": deleted_total,
                        "status": "deleting"
                    }));
                }
            }
        }
    }
    let _ = window.emit("deletion_complete", ());
    Ok(())
}
