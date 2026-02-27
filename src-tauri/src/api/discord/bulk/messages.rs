// src-tauri/src/api/discord/bulk/messages.rs

use crate::api::rate_limiter::ApiHandle;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::op_manager::{OperationManager, OperationState};
use crate::core::vault::Vault;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager, Window};

#[allow(dead_code)]
#[derive(serde::Deserialize, Clone)]
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

    let api_handle = app_handle.state::<ApiHandle>().inner();
    let op_manager = app_handle.state::<OperationManager>().inner();
    op_manager.state.prepare();
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    Logger::info(
        &app_handle,
        &format!(
            "[OP] Concurrency-enabled purge started for {} nodes",
            options.channel_ids.len()
        ),
        None,
    );

    let (tx, mut rx) = tokio::sync::mpsc::channel::<usize>(options.channel_ids.len());

    for (i, channel_id) in options.channel_ids.iter().cloned().enumerate() {
        let app_clone = app_handle.clone();
        let window_clone = window.clone();
        let opt_clone = options.clone();
        let token_clone = token.clone();
        let uid_clone = current_user_id.clone();
        let api_clone = api_handle.clone();
        let state_clone = op_manager.state.clone();
        let tx_clone = tx.clone();

        tauri::async_runtime::spawn(async move {
            let count = process_channel_task(
                &app_clone,
                &window_clone,
                &opt_clone,
                &channel_id,
                i,
                &token_clone,
                is_bearer,
                &uid_clone,
                &api_clone,
                &state_clone,
            )
            .await
            .unwrap_or(0);
            let _ = tx_clone.send(count).await;
        });
    }

    drop(tx);
    let mut total_deleted = 0;
    while let Some(count) = rx.recv().await {
        total_deleted += count;
    }

    op_manager.state.reset();
    let _ = window.emit("deletion_complete", total_deleted);
    Logger::info(
        &app_handle,
        &format!("[OP] Purge finished. Total nullified: {}", total_deleted),
        None,
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn process_channel_task(
    _app: &AppHandle,
    window: &Window,
    options: &PurgeOptions,
    channel_id: &str,
    index: usize,
    token: &str,
    is_bearer: bool,
    user_id: &str,
    api: &ApiHandle,
    state: &OperationState,
) -> Result<usize, AppError> {
    state.wait_if_paused().await;
    if state.should_abort.load(Ordering::SeqCst) {
        return Ok(0);
    }

    let mut deleted = 0;

    // 1. Search API Pass (Optimized)
    if !options.simulation {
        let mut search_url = format!(
            "https://discord.com/api/v9/channels/{}/messages/search?author_id={}",
            channel_id, user_id
        );
        if let Some(q) = &options.search_query {
            search_url.push_str(&format!("&content={}", urlencoding::encode(q)));
        }

        if let Ok(res) = api
            .send_request_json(
                reqwest::Method::GET,
                &search_url,
                None,
                token,
                is_bearer,
                None,
            )
            .await
            && let Some(messages) = res["messages"].as_array()
        {
            for batch in messages {
                if let Some(msg_array) = batch.as_array() {
                    for msg in msg_array {
                        state.wait_if_paused().await;
                        if state.should_abort.load(Ordering::SeqCst) {
                            break;
                        }

                        if let Some(id) = msg["id"].as_str() {
                            let del_url = format!(
                                "https://discord.com/api/v9/channels/{}/messages/{}",
                                channel_id, id
                            );
                            if api
                                .send_request_json(
                                    reqwest::Method::DELETE,
                                    &del_url,
                                    None,
                                    token,
                                    is_bearer,
                                    None,
                                )
                                .await
                                .is_ok()
                            {
                                deleted += 1;
                                let _ = window.emit(
                                    "deletion_progress",
                                    serde_json::json!({
                                        "current": index + 1,
                                        "total": 0,
                                        "id": channel_id,
                                        "deleted_count": deleted,
                                        "status": "purging_optimized"
                                    }),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Linear Scan Pass (Safety Net)
    let mut last_id: Option<String> = None;
    'message_loop: loop {
        state.wait_if_paused().await;
        if state.should_abort.load(Ordering::SeqCst) {
            break 'message_loop;
        }

        let mut url = format!(
            "https://discord.com/api/v9/channels/{}/messages?limit=100",
            channel_id
        );
        if let Some(id) = &last_id {
            url.push_str(&format!("&before={}", id));
        }

        let res = api
            .send_request_json(reqwest::Method::GET, &url, None, token, is_bearer, None)
            .await;
        let messages: Vec<serde_json::Value> = match res {
            Ok(v) => serde_json::from_value(v).map_err(AppError::from)?,
            _ => break,
        };

        if messages.is_empty() {
            break;
        }
        last_id = messages
            .last()
            .and_then(|m| m["id"].as_str().map(|s| s.to_string()));

        for msg in messages {
            state.wait_if_paused().await;
            if state.should_abort.load(Ordering::SeqCst) {
                break 'message_loop;
            }

            if msg["author"]["id"].as_str() != Some(user_id) {
                continue;
            }

            let msg_id = msg["id"].as_str().unwrap_or_default();
            let matches = options
                .search_query
                .as_ref()
                .map(|q| {
                    msg["content"]
                        .as_str()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&q.to_lowercase())
                })
                .unwrap_or(true);

            if !options.simulation && matches {
                let del_url = format!(
                    "https://discord.com/api/v9/channels/{}/messages/{}",
                    channel_id, msg_id
                );
                if api
                    .send_request_json(
                        reqwest::Method::DELETE,
                        &del_url,
                        None,
                        token,
                        is_bearer,
                        None,
                    )
                    .await
                    .is_ok()
                {
                    deleted += 1;
                    let _ = window.emit(
                        "deletion_progress",
                        serde_json::json!({
                            "current": index + 1,
                            "total": 0,
                            "id": channel_id,
                            "deleted_count": deleted,
                            "status": "purging_scan"
                        }),
                    );
                }
            } else if matches {
                deleted += 1;
            }
        }
    }

    Ok(deleted)
}
