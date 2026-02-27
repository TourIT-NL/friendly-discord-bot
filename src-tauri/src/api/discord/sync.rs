// src-tauri/src/api/discord/sync.rs

use super::types::{Channel, Guild, Relationship};
use crate::api::rate_limiter::ApiHandle;
use crate::core::cache::CacheManager;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::vault::Vault;
use tauri::{AppHandle, Emitter, Manager, Window};

#[tauri::command]
pub async fn fetch_guilds(app_handle: AppHandle) -> Result<Vec<Guild>, AppError> {
    let identity = Vault::get_active_identity(&app_handle)?;
    let token = identity.token;
    let is_bearer = identity.is_oauth;
    let identity_id = identity.id;
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

    let guilds: Vec<Guild> = serde_json::from_value(json).map_err(AppError::from)?;
    let _ = CacheManager::upsert_guilds(&app_handle, &identity_id, &guilds);
    Ok(guilds)
}

#[tauri::command]
pub async fn fetch_channels(
    app_handle: AppHandle,
    guild_id: Option<String>,
) -> Result<Vec<Channel>, AppError> {
    let identity = Vault::get_active_identity(&app_handle)?;
    let token = identity.token;
    let is_bearer = identity.is_oauth;
    let identity_id = identity.id;
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
        let filtered: Vec<Channel> = channels
            .into_iter()
            .filter(|c| [0, 5, 11, 12, 15].contains(&c.channel_type))
            .collect();
        let _ = CacheManager::upsert_channels(&app_handle, &identity_id, Some(&gid), &filtered);
        Ok(filtered)
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
        let _ = CacheManager::upsert_channels(&app_handle, &identity_id, None, &result);
        Ok(result)
    }
}

#[tauri::command]
pub async fn fetch_relationships(app_handle: AppHandle) -> Result<Vec<Relationship>, AppError> {
    let identity = Vault::get_active_identity(&app_handle)?;
    let token = identity.token;
    let is_bearer = identity.is_oauth;
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
    let identity = Vault::get_active_identity(&app_handle)?;
    let token = identity.token;
    let is_bearer = identity.is_oauth;
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

#[tauri::command]
pub async fn search_local_cache(
    app_handle: AppHandle,
    query: String,
) -> Result<Vec<serde_json::Value>, AppError> {
    CacheManager::search_messages(&app_handle, &query)
}

#[tauri::command]
pub async fn start_deep_scan(
    app_handle: AppHandle,
    window: Window,
    channel_ids: Vec<String>,
) -> Result<(), AppError> {
    let identity = Vault::get_active_identity(&app_handle)?;
    let token = identity.token;
    let is_bearer = identity.is_oauth;
    let identity_id = identity.id;
    let api_handle = app_handle.state::<ApiHandle>();

    for (i, channel_id) in channel_ids.iter().enumerate() {
        Logger::info(
            &app_handle,
            &format!("[SCAN] Hydrating cache for channel {}", channel_id),
            None,
        );
        let mut last_id: Option<String> = None;
        loop {
            let mut url = format!(
                "https://discord.com/api/v9/channels/{}/messages?limit=100",
                channel_id
            );
            if let Some(id) = &last_id {
                url.push_str(&format!("&before={}", id));
            }

            let messages: Vec<serde_json::Value> = match api_handle
                .send_request_json(reqwest::Method::GET, &url, None, &token, is_bearer, None)
                .await
            {
                Ok(v) => serde_json::from_value(v).map_err(AppError::from)?,
                Err(_) => break,
            };

            if messages.is_empty() {
                break;
            }
            last_id = messages
                .last()
                .and_then(|m| m["id"].as_str().map(|s| s.to_string()));

            for mut msg in messages {
                if let Some(obj) = msg.as_object_mut() {
                    obj.insert("channel_id".to_string(), serde_json::json!(channel_id));
                }
                let _ = CacheManager::upsert_message(&app_handle, &identity_id, &msg);
            }
            let _ = window.emit("scan_progress", serde_json::json!({ "current": i + 1, "total": channel_ids.len(), "channel_id": channel_id }));
        }
    }
    Ok(())
}
