// src-tauri/src/api/discord/bulk/messages.rs

use crate::api::rate_limiter::{ApiHandle, types::ApiResponseContent};
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
                false,
            )
            .await;

        let channel_type = match channel_info_res {
            Ok(ApiResponseContent::Json(value)) => value["type"].as_u64().unwrap_or(0),
            _ => {
                Logger::warn(
                    &app_handle,
                    &format!("[OP] Failed to get channel info for {}", channel_id),
                    None,
                );
                continue;
            }
        };

        if channel_type == 0 {
            Logger::warn(
                &app_handle,
                &format!(
                    "[OP] Skipping guild text channel {} for message purge",
                    channel_id
                ),
                None,
            );
            continue;
        }

        let mut last_message_id: Option<String> = None;
        let mut scanned_in_channel = 0;
        let mut consecutive_failures = 0;

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

            let response_content = api_handle
                .send_request(reqwest::Method::GET, &url, None, &token, is_bearer, false)
                .await;

            let messages: Vec<serde_json::Value> = match response_content {
                Ok(ApiResponseContent::Json(value)) => {
                    serde_json::from_value(value).map_err(AppError::from)?
                }
                _ => {
                    consecutive_failures += 1;
                    if consecutive_failures > 3 {
                        break 'message_loop;
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
                if author_id != current_user_id {
                    continue;
                }

                let msg_id = msg["id"].as_str().unwrap_or_default();
                let content = msg["content"].as_str().unwrap_or_default();
                let timestamp = chrono::DateTime::parse_from_rfc3339(
                    msg["timestamp"].as_str().unwrap_or_default(),
                )
                .map(|dt| dt.timestamp_millis() as u64)
                .unwrap_or(0);

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

                let has_attachments = msg["attachments"]
                    .as_array()
                    .map(|a| !a.is_empty())
                    .unwrap_or(false);
                if options.only_attachments && !has_attachments {
                    continue;
                }

                let matches_query = if let Some(query) = &options.search_query {
                    content.to_lowercase().contains(&query.to_lowercase())
                } else {
                    true
                };

                if !options.simulation && matches_query {
                    // Purge reactions if requested
                    if options.purge_reactions
                        && let Some(reactions) = msg["reactions"].as_array()
                    {
                        for r in reactions {
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
                                        false,
                                    )
                                    .await;
                            }
                        }
                    }

                    let del_url = format!(
                        "https://discord.com/api/v9/channels/{}/messages/{}",
                        channel_id, msg_id
                    );
                    if api_handle
                        .send_request(
                            reqwest::Method::DELETE,
                            &del_url,
                            None,
                            &token,
                            is_bearer,
                            false,
                        )
                        .await
                        .is_ok()
                    {
                        deleted_total += 1;
                    }
                } else if matches_query {
                    deleted_total += 1;
                }
            }
        } // End of message_loop

        if options.close_empty_dms && !options.simulation {
            // Optional: Close DM node logic
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
