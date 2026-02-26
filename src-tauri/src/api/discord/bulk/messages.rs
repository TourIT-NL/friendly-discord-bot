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
    op_manager.state.prepare();
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    let mut deleted_total = 0;

    'channel_loop: for (i, channel_id) in options.channel_ids.iter().enumerate() {
        op_manager.state.wait_if_paused().await;
        if op_manager.state.should_abort.load(Ordering::SeqCst) {
            break;
        }

        let info_url = format!("https://discord.com/api/v9/channels/{}", channel_id);
        let info_res = api_handle
            .send_request(
                reqwest::Method::GET,
                &info_url,
                None,
                &token,
                is_bearer,
                false,
                None,
                None,
                None,
                None,
            )
            .await;
        let c_type = match info_res {
            Ok(ApiResponseContent::Json(v)) => v["type"].as_u64().unwrap_or(0),
            _ => continue,
        };

        if c_type == 0 {
            continue;
        }

        let mut last_id: Option<String> = None;
        'message_loop: loop {
            op_manager.state.wait_if_paused().await;
            if op_manager.state.should_abort.load(Ordering::SeqCst) {
                break 'channel_loop;
            }

            let mut url = format!(
                "https://discord.com/api/v9/channels/{}/messages?limit=100",
                channel_id
            );
            if let Some(id) = &last_id {
                url.push_str(&format!("&before={}", id));
            }

            let res = api_handle
                .send_request(
                    reqwest::Method::GET,
                    &url,
                    None,
                    &token,
                    is_bearer,
                    false,
                    None,
                    None,
                    None,
                    None,
                )
                .await;
            let messages: Vec<serde_json::Value> = match res {
                Ok(ApiResponseContent::Json(v)) => {
                    serde_json::from_value(v).map_err(AppError::from)?
                }
                _ => break,
            };

            if messages.is_empty() {
                break;
            }
            last_id = messages
                .last()
                .and_then(|m| m["id"].as_str().map(|s| s.to_string()));

            for msg in messages {
                op_manager.state.wait_if_paused().await;
                if op_manager.state.should_abort.load(Ordering::SeqCst) {
                    break 'channel_loop;
                }

                if msg["author"]["id"].as_str() != Some(&current_user_id) {
                    continue;
                }

                let msg_id = msg["id"].as_str().unwrap_or_default();
                let content = msg["content"].as_str().unwrap_or_default();
                let ts = chrono::DateTime::parse_from_rfc3339(
                    msg["timestamp"].as_str().unwrap_or_default(),
                )
                .map(|dt| dt.timestamp_millis() as u64)
                .unwrap_or(0);

                if let Some(start) = options.start_time {
                    if ts < start {
                        if last_id.is_some() {
                            break 'message_loop;
                        } else {
                            continue;
                        }
                    }
                }
                if let Some(end) = options.end_time {
                    if ts > end {
                        continue;
                    }
                }

                let has_att = msg["attachments"]
                    .as_array()
                    .map(|a| !a.is_empty())
                    .unwrap_or(false);
                if options.only_attachments && !has_att {
                    continue;
                }

                let matches = options
                    .search_query
                    .as_ref()
                    .map(|q| content.to_lowercase().contains(&q.to_lowercase()))
                    .unwrap_or(true);

                if !options.simulation && matches {
                    if options.purge_reactions {
                        if let Some(reactions) = msg["reactions"].as_array() {
                            for r in reactions {
                                if r["me"].as_bool().unwrap_or(false) {
                                    let emoji = r["emoji"]["name"].as_str().unwrap_or("");
                                    let eid = r["emoji"]["id"].as_str().unwrap_or("");
                                    let param = if eid.is_empty() {
                                        emoji.to_string()
                                    } else {
                                        format!("{}:{}", emoji, eid)
                                    };
                                    let url = format!(
                                        "https://discord.com/api/v9/channels/{}/messages/{}/reactions/{}/@me",
                                        channel_id, msg_id, param
                                    );
                                    let _ = api_handle
                                        .send_request(
                                            reqwest::Method::DELETE,
                                            &url,
                                            None,
                                            &token,
                                            is_bearer,
                                            false,
                                            None,
                                            None,
                                            None,
                                            None,
                                        )
                                        .await;
                                }
                            }
                        }
                    }

                    let url = format!(
                        "https://discord.com/api/v9/channels/{}/messages/{}",
                        channel_id, msg_id
                    );
                    if api_handle
                        .send_request(
                            reqwest::Method::DELETE,
                            &url,
                            None,
                            &token,
                            is_bearer,
                            false,
                            None,
                            None,
                            None,
                            None,
                        )
                        .await
                        .is_ok()
                    {
                        deleted_total += 1;
                    }
                } else if matches {
                    deleted_total += 1;
                }
            }
        }
    }

    op_manager.state.reset();
    let _ = window.emit("deletion_complete", ());
    Ok(())
}
