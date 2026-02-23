// src-tauri/src/api/discord/bulk/messages.rs

use crate::api::rate_limiter::ApiHandle;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::op_manager::OperationManager;
use crate::core::vault::Vault;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, Window};

#[derive(serde::Deserialize)]
pub struct PurgeOptions {
    #[serde(alias = "channelIds")]
    pub channel_ids: Vec<String>,
    #[serde(alias = "startTime")]
    pub start_time: Option<u64>,
    #[serde(alias = "endTime")]
    pub end_time: Option<u64>,
    #[serde(alias = "searchQuery")]
    pub search_query: Option<String>,
    #[serde(alias = "purgeReactions")]
    pub purge_reactions: bool,
    pub simulation: bool,
    #[serde(alias = "onlyAttachments")]
    pub only_attachments: bool,
    #[serde(alias = "closeEmptyDms")]
    pub close_empty_dms: bool,
}

#[tauri::command]
pub async fn bulk_delete_messages(
    app_handle: AppHandle,
    window: Window,
    options: PurgeOptions,
) -> Result<(), AppError> {
    let identity = Vault::get_active_identity(&app_handle)?;
    let token = identity.token;
    let is_bearer = identity.is_oauth;
    let current_user_id = identity.id;

    let api_handle = app_handle.state::<ApiHandle>();
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.prepare(); // Ensure clean state
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    Logger::info(
        &app_handle,
        &format!(
            "[OP] Destructive purge initialized for {} nodes (User: {}, Sim: {}). Is OAuth token: {}",
            options.channel_ids.len(),
            current_user_id,
            options.simulation,
            is_bearer
        ),
        None,
    );

    let mut deleted_total = 0;

    'channel_loop: for (i, channel_id) in options.channel_ids.iter().enumerate() {
        // Check pause/abort before processing new channel
        op_manager.state.wait_if_paused().await;
        if op_manager.state.should_abort.load(Ordering::SeqCst) {
            break 'channel_loop;
        }

        let channel_info_url = format!("https://discord.com/api/v9/channels/{}", channel_id);

        // Check pause/abort before channel info fetch
        op_manager.state.wait_if_paused().await;
        if op_manager.state.should_abort.load(Ordering::SeqCst) {
            break 'channel_loop;
        }

        let channel_info_res = api_handle
            .send_request(
                reqwest::Method::GET,
                &channel_info_url,
                None,
                &token,
                is_bearer,
            )
            .await;

        let channel_type = match channel_info_res {
            Ok(value) => value["type"].as_u64().unwrap_or(0),
            Err(e) => {
                if e.error_code.contains("403") || e.user_message.contains("403") {
                    Logger::warn(
                        &app_handle,
                        &format!(
                            "[OP] Access denied (403) for channel {}. Assuming guild channel and skipping.",
                            channel_id
                        ),
                        None,
                    );
                } else {
                    Logger::warn(
                        &app_handle,
                        &format!(
                            "[OP] Failed to get channel info for channel {}: {}. Skipping channel.",
                            channel_id, e.user_message
                        ),
                        None,
                    );
                }
                continue;
            }
        };

        Logger::info(
            &app_handle,
            &format!(
                "[OP] Channel {} type: {} (Is OAuth token: {})",
                channel_id, channel_type, is_bearer
            ),
            None,
        );

        if channel_type == 0 {
            Logger::warn(
                &app_handle,
                &format!(
                    "[OP] Skipping guild text channel {} for message purge due to user token limitations.",
                    channel_id
                ),
                None,
            );
            continue; // Skip this channel
        }

        let mut last_message_id: Option<String> = None;
        let mut consecutive_failures = 0;
        let mut scanned_in_channel = 0;

        // Emit initial status for this channel
        let _ = window.emit(
            "deletion_progress",
            serde_json::json!({
                "current": i + 1,
                "total": options.channel_ids.len(),
                "id": channel_id,
                "deleted_count": deleted_total,
                "scanned_count": 0,
                "status": "scanning"
            }),
        );

        'message_loop: loop {
            op_manager.state.wait_if_paused().await;
            if op_manager.state.should_abort.load(Ordering::SeqCst) {
                break 'channel_loop;
            }

            let mut url = format!(
                "https://discord.com/api/v9/channels/{}/messages?limit=100",
                channel_id
            );
            if let Some(before) = &last_message_id {
                url.push_str(&format!("&before={}", before));
            }

            let response_value = api_handle
                .send_request(reqwest::Method::GET, &url, None, &token, is_bearer)
                .await;

            let messages: Vec<serde_json::Value> = match response_value {
                Ok(value) => serde_json::from_value(value).map_err(AppError::from)?,
                Err(e) => {
                    Logger::warn(
                        &app_handle,
                        &format!(
                            "[OP] Failed to fetch messages from channel {}: {}",
                            channel_id, e.user_message
                        ),
                        None,
                    );
                    consecutive_failures += 1;
                    if consecutive_failures > 3 {
                        break 'message_loop;
                    }
                    // Check abort before sleeping
                    if op_manager.state.should_abort.load(Ordering::SeqCst) {
                        break 'channel_loop;
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            if messages.is_empty() {
                break 'message_loop;
            }

            scanned_in_channel += messages.len();
            last_message_id = messages
                .last()
                .and_then(|m| m["id"].as_str().map(|s| s.to_string()));

            // Update status after fetching batch
            let _ = window.emit(
                "deletion_progress",
                serde_json::json!({
                    "current": i + 1,
                    "total": options.channel_ids.len(),
                    "id": channel_id,
                    "deleted_count": deleted_total,
                    "scanned_count": scanned_in_channel,
                    "status": "processing batch"
                }),
            );

            for msg in messages {
                op_manager.state.wait_if_paused().await;
                if op_manager.state.should_abort.load(Ordering::SeqCst) {
                    break 'channel_loop;
                }

                let author_id = msg["author"]["id"].as_str().unwrap_or_default();
                let sys_user_id = msg["user"]["id"].as_str().unwrap_or_default();

                if author_id != current_user_id && sys_user_id != current_user_id {
                    continue;
                }

                let msg_id = msg["id"].as_str().unwrap_or_default();
                let content = msg["content"].as_str().unwrap_or_default();
                let timestamp = chrono::DateTime::parse_from_rfc3339(
                    msg["timestamp"].as_str().unwrap_or_default(),
                )
                .map(|dt| dt.timestamp_millis() as u64)
                .unwrap_or(0);

                let matches_query = if let Some(query) = &options.search_query {
                    content.to_lowercase().contains(&query.to_lowercase())
                } else {
                    true
                };
                let has_attachments = msg["attachments"]
                    .as_array()
                    .is_some_and(|arr| !arr.is_empty());

                if let Some(start) = options.start_time
                    && timestamp < start
                {
                    if last_message_id.is_some() {
                        break 'message_loop;
                    } else {
                        continue;
                    }
                }
                if let Some(end) = options.end_time
                    && timestamp > end
                {
                    continue;
                }

                if options.only_attachments && !has_attachments {
                    continue;
                }

                if !options.simulation {
                    if options.purge_reactions
                        && let Some(reactions) = msg["reactions"].as_array()
                    {
                        for r in reactions {
                            op_manager.state.wait_if_paused().await;
                            if op_manager.state.should_abort.load(Ordering::SeqCst) {
                                break 'channel_loop;
                            }
                            if r["me"].as_bool().unwrap_or(false) {
                                let emoji = r["emoji"]["name"].as_str().unwrap_or("");
                                let emoji_id = r["emoji"]["id"].as_str().unwrap_or("");
                                let emoji_param = if emoji_id.is_empty() {
                                    emoji.to_string()
                                } else {
                                    format!("{}:{}", emoji, emoji_id)
                                };
                                let react_url = format!(
                                    "https://discord.com/api/v9/channels/{}/messages/{}/reactions/{}/@me",
                                    channel_id, msg_id, emoji_param
                                );
                                let _ = api_handle
                                    .send_request(
                                        reqwest::Method::DELETE,
                                        &react_url,
                                        None,
                                        &token,
                                        is_bearer,
                                    )
                                    .await;
                            }
                        }
                    }

                    if matches_query {
                        op_manager.state.wait_if_paused().await;
                        if op_manager.state.should_abort.load(Ordering::SeqCst) {
                            break 'channel_loop;
                        }

                        let del_url = format!(
                            "https://discord.com/api/v9/channels/{}/messages/{}",
                            channel_id, msg_id
                        );
                        let del_res = api_handle
                            .send_request(
                                reqwest::Method::DELETE,
                                &del_url,
                                None,
                                &token,
                                is_bearer,
                            )
                            .await;
                        if del_res.is_ok() {
                            deleted_total += 1;
                            // Emit progress on successful deletion for responsiveness
                            let _ = window.emit(
                                "deletion_progress",
                                serde_json::json!({
                                    "current": i + 1,
                                    "total": options.channel_ids.len(),
                                    "id": channel_id,
                                    "deleted_count": deleted_total,
                                    "scanned_count": scanned_in_channel,
                                    "status": "purging"
                                }),
                            );
                        }
                    }
                } else if matches_query {
                    deleted_total += 1;
                    // Emit progress in simulation
                    let _ = window.emit(
                        "deletion_progress",
                        serde_json::json!({
                            "current": i + 1,
                            "total": options.channel_ids.len(),
                            "id": channel_id,
                            "deleted_count": deleted_total,
                            "scanned_count": scanned_in_channel,
                            "status": "simulating"
                        }),
                    );
                }
            }
        } // End of message_loop

        // Check abort/pause before DM close
        op_manager.state.wait_if_paused().await;
        if op_manager.state.should_abort.load(Ordering::SeqCst) {
            break 'channel_loop;
        }

        if options.close_empty_dms && !options.simulation {
            let chan_url = format!("https://discord.com/api/v9/channels/{}", channel_id);
            if let Ok(chan_val) = api_handle
                .send_request(reqwest::Method::GET, &chan_url, None, &token, is_bearer)
                .await
            {
                let chan_type = chan_val["type"].as_u64().unwrap_or(0);
                if chan_type == 1 || chan_type == 3 {
                    let check_url = format!(
                        "https://discord.com/api/v9/channels/{}/messages?limit=1",
                        channel_id
                    );
                    if let Ok(check_val) = api_handle
                        .send_request(reqwest::Method::GET, &check_url, None, &token, is_bearer)
                        .await
                        && check_val.as_array().map(|a| a.is_empty()).unwrap_or(false)
                    {
                        Logger::info(
                            &app_handle,
                            &format!("[OP] Closing empty DM node {}", channel_id),
                            None,
                        );
                        let _ = api_handle
                            .send_request(
                                reqwest::Method::DELETE,
                                &chan_url,
                                None,
                                &token,
                                is_bearer,
                            )
                            .await;
                    }
                }
            }
        }
    } // End of channel loop

    op_manager.state.reset();
    let _ = window.emit("deletion_complete", ());
    Logger::info(
        &app_handle,
        &format!(
            "[OP] Destructive purge complete. Items nullified: {}",
            deleted_total
        ),
        None,
    );
    Ok(())
}
