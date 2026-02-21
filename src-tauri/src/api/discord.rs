// src-tauri/src/api/discord.rs

use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Manager, Emitter};
use crate::api::rate_limiter::ApiHandle;
use crate::core::error::AppError;
use crate::core::vault::Vault;
use crate::core::op_manager::OperationManager;
use crate::core::logger::Logger;
use std::time::Duration;
use std::sync::atomic::Ordering;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Guild {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Channel {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub channel_type: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Relationship {
    pub id: String,
    pub nickname: Option<String>,
    pub user: serde_json::Value,
    #[serde(rename = "type")]
    pub rel_type: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OperationStatus {
    pub is_running: bool,
    pub is_paused: bool,
    pub should_abort: bool,
}

#[tauri::command]
pub async fn fetch_guilds(app_handle: AppHandle) -> Result<Vec<Guild>, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    Logger::info(&app_handle, &format!("[SYNC] Fetching guilds (OAuth: {})...", is_bearer), None);
    let response = api_handle.send_request(reqwest::Method::GET, "https://discord.com/api/v9/users/@me/guilds", None, &token, is_bearer).await?;
    if !response.status().is_success() {
        Logger::error(&app_handle, "[SYNC] Guild fetch failed", Some(serde_json::json!({ "status": response.status().as_u16() })));
        return Err(AppError { user_message: "Sync failed.".into(), ..Default::default() });
    }
    let guilds: Vec<Guild> = response.json().await?;
    Logger::debug(&app_handle, &format!("[SYNC] Discovered {} guilds", guilds.len()), None);
    Ok(guilds)
}

#[tauri::command]
pub async fn fetch_channels(app_handle: AppHandle, guild_id: Option<String>) -> Result<Vec<Channel>, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    
    if let Some(gid) = guild_id {
        Logger::info(&app_handle, &format!("[SYNC] Mapping channels for guild {}", gid), None);
        let response = api_handle.send_request(reqwest::Method::GET, &format!("https://discord.com/api/v9/guilds/{}/channels", gid), None, &token, is_bearer).await?;
        if !response.status().is_success() { 
            Logger::error(&app_handle, "[SYNC] Channel mapping failed", Some(serde_json::json!({ "guild": gid, "status": response.status().as_u16() })));
            return Err(AppError { user_message: "Channel mapping failed.".into(), ..Default::default() }); 
        }
        let channels: Vec<Channel> = response.json().await?;
        let filtered: Vec<Channel> = channels.into_iter().filter(|c| c.channel_type == 0 || c.channel_type == 11 || c.channel_type == 12).collect();
        Logger::debug(&app_handle, &format!("[SYNC] Mapped {} valid buffers", filtered.len()), None);
        Ok(filtered)
    } else {
        Logger::info(&app_handle, "[SYNC] Fetching DM buffers...", None);
        if is_bearer { return Err(AppError { user_message: "DMs restricted in Official Gate.".into(), ..Default::default() }); }
        let response = api_handle.send_request(reqwest::Method::GET, "https://discord.com/api/v9/users/@me/channels", None, &token, is_bearer).await?;
        if !response.status().is_success() { return Err(AppError { user_message: "DM sync failed.".into(), ..Default::default() }); }
        let channels: Vec<serde_json::Value> = response.json().await?;
        let mut result = Vec::new();
        for ch in channels {
            let ch_type = ch["type"].as_u64().unwrap_or(0);
            if ch_type == 1 || ch_type == 3 {
                let name = if ch_type == 1 {
                    ch["recipients"].as_array().and_then(|r| r.get(0)).and_then(|u| u["username"].as_str()).map(|s| format!("DM with {}", s))
                } else {
                    ch["name"].as_str().map(|s| s.to_string()).or_else(|| Some("Unnamed Group DM".to_string()))
                };
                result.push(Channel { id: ch["id"].as_str().unwrap_or_default().to_string(), name, channel_type: ch_type as u8 });
            }
        }
        Logger::debug(&app_handle, &format!("[SYNC] Mapped {} private buffers", result.len()), None);
        Ok(result)
    }
}

#[tauri::command]
pub async fn fetch_relationships(app_handle: AppHandle) -> Result<Vec<Relationship>, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    if is_bearer { return Err(AppError { user_message: "Relationships restricted in Official Gate.".into(), ..Default::default() }); }
    Logger::info(&app_handle, "[SYNC] Fetching relationships...", None);
    let response = api_handle.send_request(reqwest::Method::GET, "https://discord.com/api/v9/users/@me/relationships", None, &token, is_bearer).await?;
    if !response.status().is_success() { return Err(AppError { user_message: "Identity sync failed.".into(), ..Default::default() }); }
    let rels: Vec<Relationship> = response.json().await?;
    Logger::debug(&app_handle, &format!("[SYNC] Found {} linked identities", rels.len()), None);
    Ok(rels)
}

#[tauri::command]
pub async fn bulk_remove_relationships(app_handle: AppHandle, window: tauri::Window, user_ids: Vec<String>) -> Result<(), AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    Logger::info(&app_handle, &format!("[OP] Starting bulk relationship severance for {} identities", user_ids.len()), None);

    for (i, user_id) in user_ids.iter().enumerate() {
        op_manager.state.wait_if_paused().await;
        if op_manager.state.should_abort.load(Ordering::SeqCst) { 
            Logger::warn(&app_handle, "[OP] Relationship severance aborted", None);
            break; 
        }

        let url = format!("https://discord.com/api/v9/users/@me/relationships/{}", user_id);
        let _ = api_handle.send_request(reqwest::Method::DELETE, &url, None, &token, is_bearer).await;
        let _ = window.emit("relationship_progress", serde_json::json!({ "current": i + 1, "total": user_ids.len(), "id": user_id, "status": "severing" }));
    }
    op_manager.state.reset();
    let _ = window.emit("relationship_complete", ());
    Logger::info(&app_handle, "[OP] Identity purge complete", None);
    Ok(())
}

#[tauri::command]
pub async fn bulk_leave_guilds(app_handle: AppHandle, window: tauri::Window, guild_ids: Vec<String>) -> Result<(), AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    Logger::info(&app_handle, &format!("[OP] Connection severance initiated for {} nodes", guild_ids.len()), None);

    for (i, guild_id) in guild_ids.iter().enumerate() {
        op_manager.state.wait_if_paused().await;
        if op_manager.state.should_abort.load(Ordering::SeqCst) { 
            Logger::warn(&app_handle, "[OP] Node severance aborted", None);
            break; 
        }

        let url = format!("https://discord.com/api/v9/users/@me/guilds/{}", guild_id);
        let _ = api_handle.send_request(reqwest::Method::DELETE, &url, None, &token, is_bearer).await;
        let _ = window.emit("leave_progress", serde_json::json!({ "current": i + 1, "total": guild_ids.len(), "id": guild_id, "status": "severing" }));
    }
    op_manager.state.reset();
    let _ = window.emit("leave_complete", ());
    Logger::info(&app_handle, "[OP] Node severance complete", None);
    Ok(())
}

#[tauri::command]
pub async fn bulk_delete_messages(
    app_handle: AppHandle,
    window: tauri::Window,
    channel_ids: Vec<String>,
    start_time: Option<u64>,
    end_time: Option<u64>,
    search_query: Option<String>,
    purge_reactions: bool,
    simulation: bool,
    only_attachments: bool,
) -> Result<(), AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    Logger::info(&app_handle, &format!("[OP] Initializing destructive purge for {} buffers (Sim: {})", channel_ids.len(), simulation), None);

    let mut deleted_total = 0;

    for (i, channel_id) in channel_ids.iter().enumerate() {
        let mut last_message_id: Option<String> = None;
        let mut consecutive_failures = 0;

        Logger::debug(&app_handle, &format!("[OP] Purging node {}", channel_id), None);

        'message_loop: loop {
            op_manager.state.wait_if_paused().await;
            if op_manager.state.should_abort.load(Ordering::SeqCst) { break; }

            let mut url = format!("https://discord.com/api/v9/channels/{}/messages?limit=100", channel_id);
            if let Some(before) = &last_message_id { url.push_str(&format!("&before={}", before)); }

            let response = api_handle.send_request(reqwest::Method::GET, &url, None, &token, is_bearer).await?;
            if !response.status().is_success() { 
                consecutive_failures += 1;
                Logger::warn(&app_handle, &format!("[OP] Chunk fetch failed for channel {} ({}/3)", channel_id, consecutive_failures), None);
                if consecutive_failures > 3 { break; }
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue; 
            }
            consecutive_failures = 0;

            let messages: Vec<serde_json::Value> = response.json().await?;
            if messages.is_empty() { break; }
            last_message_id = messages.last().and_then(|m| m["id"].as_str().map(|s| s.to_string()));

            for msg in messages {
                op_manager.state.wait_if_paused().await;
                if op_manager.state.should_abort.load(Ordering::SeqCst) { break 'message_loop; }

                let msg_id = msg["id"].as_str().unwrap_or_default();
                let content = msg["content"].as_str().unwrap_or_default();
                let timestamp = chrono::DateTime::parse_from_rfc3339(msg["timestamp"].as_str().unwrap_or_default()).map(|dt| dt.timestamp_millis() as u64).unwrap_or(0);
                
                let matches_query = if let Some(query) = &search_query { content.to_lowercase().contains(&query.to_lowercase()) } else { true };
                let has_attachments = msg["attachments"].as_array().map_or(false, |arr| !arr.is_empty());

                if let Some(start) = start_time { if timestamp < start { if last_message_id.is_some() { break 'message_loop; } else { continue; } }}
                if let Some(end) = end_time { if timestamp > end { continue; }}

                if only_attachments && !has_attachments { continue; }

                if !simulation {
                    if purge_reactions {
                        if let Some(reactions) = msg["reactions"].as_array() {
                            for r in reactions {
                                op_manager.state.wait_if_paused().await;
                                if op_manager.state.should_abort.load(Ordering::SeqCst) { break 'message_loop; }

                                if r["me"].as_bool().unwrap_or(false) {
                                    let emoji = r["emoji"]["name"].as_str().unwrap_or("");
                                    let emoji_id = r["emoji"]["id"].as_str().unwrap_or("");
                                    let emoji_param = if emoji_id.is_empty() { emoji.to_string() } else { format!("{}:{}", emoji, emoji_id) };
                                    let react_url = format!("https://discord.com/api/v9/channels/{}/messages/{}/reactions/{}/@me", channel_id, msg_id, emoji_param);
                                    let _ = api_handle.send_request(reqwest::Method::DELETE, &react_url, None, &token, is_bearer).await;
                                }
                            }
                        }
                    }

                    if matches_query {
                        let del_url = format!("https://discord.com/api/v9/channels/{}/messages/{}", channel_id, msg_id);
                        let del_res = api_handle.send_request(reqwest::Method::DELETE, &del_url, None, &token, is_bearer).await;
                        if let Ok(res) = del_res { if res.status().is_success() { deleted_total += 1; } }
                    }
                } else {
                    if matches_query { deleted_total += 1; }
                }

                if deleted_total % 10 == 0 {
                    let _ = window.emit("deletion_progress", serde_json::json!({ "current": i + 1, "total": channel_ids.len(), "id": channel_id, "deleted_count": deleted_total, "status": if simulation { "simulating" } else { "purging" } }));
                }
            }
        }
    }
    op_manager.state.reset();
    let _ = window.emit("deletion_complete", ());
    Logger::info(&app_handle, &format!("[OP] Destructive purge complete. Total items nullified: {}", deleted_total), None);
    Ok(())
}

#[tauri::command]
pub async fn fetch_preview_messages(app_handle: AppHandle, channel_id: String) -> Result<Vec<serde_json::Value>, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    let response = api_handle.send_request(reqwest::Method::GET, &format!("https://discord.com/api/v9/channels/{}/messages?limit=5", channel_id), None, &token, is_bearer).await?;
    if !response.status().is_success() { return Err(AppError { user_message: "Preview failed.".into(), ..Default::default() }); }
    Ok(response.json().await?)
}

#[tauri::command]
pub async fn stealth_privacy_wipe(app_handle: AppHandle) -> Result<(), AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    if is_bearer { return Err(AppError { user_message: "Stealth Mode restricted in Official Gate.".into(), ..Default::default() }); }

    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    Logger::info(&app_handle, "[STEALTH] Initiating privacy wipe...", None);

    // 1. Wipe Custom Status
    op_manager.state.wait_if_paused().await;
    if op_manager.state.should_abort.load(Ordering::SeqCst) { op_manager.state.reset(); return Ok(()); }
    Logger::debug(&app_handle, "[STEALTH] Wiping custom status", None);
    let _ = api_handle.send_request(reqwest::Method::PATCH, "https://discord.com/api/v9/users/@me/settings", Some(serde_json::json!({ "custom_status": null })), &token, is_bearer).await;

    // 2. Global DM Disable (default for new servers)
    op_manager.state.wait_if_paused().await;
    if op_manager.state.should_abort.load(Ordering::SeqCst) { op_manager.state.reset(); return Ok(()); }
    Logger::debug(&app_handle, "[STEALTH] Restricted default DMs from new servers", None);
    let _ = api_handle.send_request(reqwest::Method::PATCH, "https://discord.com/api/v9/users/@me/settings", Some(serde_json::json!({ "default_guilds_restricted": true })), &token, is_bearer).await;

    // 3. Presence Privacy
    op_manager.state.wait_if_paused().await;
    if op_manager.state.should_abort.load(Ordering::SeqCst) { op_manager.state.reset(); return Ok(()); }
    Logger::debug(&app_handle, "[STEALTH] Disabling presence game/activity tracking", None);
    let _ = api_handle.send_request(reqwest::Method::PATCH, "https://discord.com/api/v9/users/@me/settings", Some(serde_json::json!({ "show_current_game": false, "restricted_guilds": [] })), &token, is_bearer).await;

    op_manager.state.reset();
    Logger::info(&app_handle, "[STEALTH] Privacy enforcement complete", None);
    Ok(())
}

#[tauri::command]
pub async fn bury_audit_log(app_handle: AppHandle, window: tauri::Window, guild_id: String, channel_id: String) -> Result<(), AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    if is_bearer { return Err(AppError { user_message: "Audit Log Burial restricted in Official Gate.".into(), ..Default::default() }); }

    Logger::info(&app_handle, &format!("[AUDIT] Starting burial protocol in guild {}", guild_id), None);

    let original_channel_response = api_handle.send_request(reqwest::Method::GET, &format!("https://discord.com/api/v9/channels/{}", channel_id), None, &token, is_bearer).await?;
    if !original_channel_response.status().is_success() {
        op_manager.state.reset();
        return Err(AppError { user_message: "Failed to get original channel name.".into(), ..Default::default() });
    }
    let original_channel_name = original_channel_response.json::<serde_json::Value>().await?["name"].as_str().unwrap_or("general").to_string();

    for i in 0..10 {
        op_manager.state.wait_if_paused().await;
        if op_manager.state.should_abort.load(Ordering::SeqCst) { break; }

        let new_name = format!("{}-temp-{}", original_channel_name, i);
        Logger::debug(&app_handle, &format!("[AUDIT] Burial phase {}: rename to {}", i, new_name), None);
        let _ = api_handle.send_request(reqwest::Method::PATCH, &format!("https://discord.com/api/v9/channels/{}", channel_id), Some(serde_json::json!({ "name": new_name })), &token, is_bearer).await;
        
        let _ = window.emit("audit_log_progress", serde_json::json!({ "current": i + 1, "total": 20, "status": format!("Burying phase {}", i) }));
        tokio::time::sleep(Duration::from_millis(500)).await;

        let _ = api_handle.send_request(reqwest::Method::PATCH, &format!("https://discord.com/api/v9/channels/{}", channel_id), Some(serde_json::json!({ "name": original_channel_name })), &token, is_bearer).await;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    op_manager.state.reset();
    let _ = window.emit("audit_log_complete", ());
    Logger::info(&app_handle, "[AUDIT] Burial sequence finalized", None);
    Ok(())
}

#[tauri::command]
pub async fn webhook_ghosting(app_handle: AppHandle, window: tauri::Window, guild_id: String) -> Result<(), AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    if is_bearer { op_manager.state.reset(); return Err(AppError { user_message: "Webhook Ghosting restricted in Official Gate.".into(), ..Default::default() }); }

    Logger::info(&app_handle, &format!("[WEBHOOK] Detecting identity-linked hooks in {}", guild_id), None);

    let webhooks_response = api_handle.send_request(reqwest::Method::GET, &format!("https://discord.com/api/v9/guilds/{}/webhooks", guild_id), None, &token, is_bearer).await?;
    if !webhooks_response.status().is_success() {
        op_manager.state.reset();
        return Err(AppError { user_message: "Failed to fetch webhooks.".into(), ..Default::default() });
    }
    let webhooks: Vec<serde_json::Value> = webhooks_response.json().await?;

    let mut deleted_webhooks = 0;
    for webhook in &webhooks {
        op_manager.state.wait_if_paused().await;
        if op_manager.state.should_abort.load(Ordering::SeqCst) { break; }

        let webhook_id = webhook["id"].as_str().unwrap_or_default();
        let webhook_creator_id = webhook["user"]["id"].as_str().unwrap_or_default();
        let user_id_from_token = token.split('.').next().unwrap_or_default();

        if webhook_creator_id == user_id_from_token { 
            Logger::debug(&app_handle, &format!("[WEBHOOK] Nullifying hook {}", webhook_id), None);
            let _ = api_handle.send_request(reqwest::Method::DELETE, &format!("https://discord.com/api/v9/webhooks/{}", webhook_id), None, &token, is_bearer).await;
            deleted_webhooks += 1;
        }
        let _ = window.emit("webhook_progress", serde_json::json!({ "current": deleted_webhooks, "total": webhooks.len(), "status": "Ghosting" }));
    }

    op_manager.state.reset();
    let _ = window.emit("webhook_complete", ());
    Logger::info(&app_handle, &format!("[WEBHOOK] Ghosting complete. Removed {} hooks", deleted_webhooks), None);
    Ok(())
}

#[tauri::command]
pub async fn nitro_stealth_wipe(app_handle: AppHandle) -> Result<(), AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    if is_bearer { return Err(AppError { user_message: "Nitro Stealth restricted in Official Gate.".into(), ..Default::default() }); }

    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    Logger::info(&app_handle, "[NITRO] Initiating stealth wipe of Nitro-exclusive metadata", None);

    // 1. Clear About Me (Bio)
    op_manager.state.wait_if_paused().await;
    if op_manager.state.should_abort.load(Ordering::SeqCst) { op_manager.state.reset(); return Ok(()); }
    Logger::debug(&app_handle, "[NITRO] Clearing bio", None);
    let _ = api_handle.send_request(reqwest::Method::PATCH, "https://discord.com/api/v9/users/@me", Some(serde_json::json!({ "bio": "" })), &token, is_bearer).await;

    // 2. Clear Pronouns
    op_manager.state.wait_if_paused().await;
    if op_manager.state.should_abort.load(Ordering::SeqCst) { op_manager.state.reset(); return Ok(()); }
    Logger::debug(&app_handle, "[NITRO] Clearing pronouns", None);
    let _ = api_handle.send_request(reqwest::Method::PATCH, "https://discord.com/api/v9/users/@me/settings", Some(serde_json::json!({ "pronouns": "" })), &token, is_bearer).await;

    // 3. Reset Banner
    op_manager.state.wait_if_paused().await;
    if op_manager.state.should_abort.load(Ordering::SeqCst) { op_manager.state.reset(); return Ok(()); }
    Logger::debug(&app_handle, "[NITRO] Resetting profile banner", None);
    let _ = api_handle.send_request(reqwest::Method::PATCH, "https://discord.com/api/v9/users/@me", Some(serde_json::json!({ "banner": null })), &token, is_bearer).await;

    op_manager.state.reset();
    Logger::info(&app_handle, "[NITRO] Stealth wipe finalized", None);
    Ok(())
}

#[tauri::command]
pub async fn pause_operation(app_handle: AppHandle) -> Result<(), AppError> {
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.is_paused.store(true, Ordering::SeqCst);
    Logger::warn(&app_handle, "[OP] Execution loop PAUSED", None);
    Ok(())
}

#[tauri::command]
pub async fn resume_operation(app_handle: AppHandle) -> Result<(), AppError> {
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.is_paused.store(false, Ordering::SeqCst);
    Logger::info(&app_handle, "[OP] Execution loop RESUMED", None);
    Ok(())
}

#[tauri::command]
pub async fn abort_operation(app_handle: AppHandle) -> Result<(), AppError> {
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.should_abort.store(true, Ordering::SeqCst);
    op_manager.state.is_paused.store(false, Ordering::SeqCst); 
    Logger::error(&app_handle, "[OP] Execution loop ABORTED", None);
    Ok(())
}

#[tauri::command]
pub async fn get_operation_status(app_handle: AppHandle) -> Result<OperationStatus, AppError> {
    let op_manager = app_handle.state::<OperationManager>();
    Ok(OperationStatus {
        is_running: op_manager.state.is_running.load(Ordering::SeqCst),
        is_paused: op_manager.state.is_paused.load(Ordering::SeqCst),
        should_abort: op_manager.state.should_abort.load(Ordering::SeqCst),
    })
}
