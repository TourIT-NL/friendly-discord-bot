// src-tauri/src/api/discord/export.rs

use crate::api::rate_limiter::{ApiHandle, types::ApiResponseContent};
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::op_manager::OperationManager;
use crate::core::vault::Vault;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager, Window};
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

#[derive(serde::Deserialize)]
pub struct ExportOptions {
    #[serde(alias = "channelIds")]
    pub channel_ids: Vec<String>,
    pub direction: String, // "sent", "received", or "both"
    #[serde(alias = "includeAttachments")]
    pub include_attachments: bool,
    #[serde(alias = "exportFormat")]
    pub format: String, // "html" or "raw"
    #[serde(alias = "outputPath")]
    pub output_path: String,
}

#[derive(serde::Serialize, Clone)]
pub struct ExportProgress {
    pub current: usize,
    pub total: usize,
    pub channel_id: String,
    pub status: String,
    pub processed_count: usize,
}

async fn download_file(
    api_handle: &ApiHandle,
    url: &str,
    save_path: &Path,
    token: &str,
    is_bearer: bool,
) -> Result<(), AppError> {
    let response_content = api_handle
        .send_request(reqwest::Method::GET, url, None, token, is_bearer, true)
        .await?;

    match response_content {
        ApiResponseContent::Bytes(bytes) => {
            let mut file = File::create(save_path)
                .map_err(|e| AppError::new(&format!("Failed to create file: {}", e), "io_error"))?;
            file.write_all(&bytes)
                .map_err(|e| AppError::new(&format!("Failed to write file: {}", e), "io_error"))?;
            Ok(())
        }
        _ => Err(AppError::new(
            "Expected raw bytes, received JSON",
            "unexpected_response_type",
        )),
    }
}

#[tauri::command]
pub async fn start_chat_html_export(
    app_handle: AppHandle,
    window: Window,
    options: ExportOptions,
) -> Result<(), AppError> {
    if options.format != "html" {
        return Err(AppError::new(
            "Only HTML format is supported for chat export.",
            "unsupported_format",
        ));
    }

    let identity = Vault::get_active_identity(&app_handle)?;
    let token = identity.token;
    let is_bearer = identity.is_oauth;
    let current_user_id = identity.id;

    let api_handle = app_handle.state::<ApiHandle>();
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.prepare();
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    let output_dir = PathBuf::from(&options.output_path);
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).map_err(|e| {
            AppError::new(&format!("Failed to create output dir: {}", e), "io_error")
        })?;
    }

    for (i, channel_id) in options.channel_ids.iter().enumerate() {
        let _ = window.emit(
            "export_progress",
            ExportProgress {
                current: i + 1,
                total: options.channel_ids.len(),
                channel_id: channel_id.to_string(),
                status: "fetching_history".to_string(),
                processed_count: 0,
            },
        );

        let mut html_content = String::from("<html><head><style>
            body { font-family: sans-serif; background: #313338; color: #dbdee1; padding: 20px; }
            .message { margin-bottom: 15px; padding: 10px; border-radius: 8px; background: #2b2d31; }
            .author { font-weight: bold; color: #f2f3f5; margin-right: 10px; }
            .timestamp { font-size: 0.8em; color: #949ba4; }
            .content { margin-top: 5px; white-space: pre-wrap; }
            .attachment { margin-top: 10px; padding: 5px; border: 1px solid #4e5058; border-radius: 4px; display: inline-block; }
        </style></head><body>");

        let mut last_id: Option<String> = None;
        let mut count = 0;
        loop {
            let mut url = format!(
                "https://discord.com/api/v9/channels/{}/messages?limit=100",
                channel_id
            );
            if let Some(id) = &last_id {
                url.push_str(&format!("&before={}", id));
            }

            let res_content = api_handle
                .send_request(reqwest::Method::GET, &url, None, &token, is_bearer, false)
                .await;
            let messages: Vec<serde_json::Value> = match res_content {
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
                let author_id = msg["author"]["id"].as_str().unwrap_or_default();
                let is_own = author_id == current_user_id;

                let should_process = match options.direction.as_str() {
                    "sent" => is_own,
                    "received" => !is_own,
                    _ => true,
                };

                if !should_process {
                    continue;
                }

                let author = msg["author"]["username"].as_str().unwrap_or("Unknown");
                let content = msg["content"].as_str().unwrap_or("");
                let ts = msg["timestamp"].as_str().unwrap_or("");

                html_content.push_str(&format!(
                    "<div class='message'><span class='author'>{}</span><span class='timestamp'>{}</span><div class='content'>{}</div>",
                    author, ts, content
                ));

                if options.include_attachments
                    && let Some(atts) = msg["attachments"].as_array()
                {
                    for att in atts {
                        let att_name = att["filename"].as_str().unwrap_or("file");
                        html_content.push_str(&format!(
                            "<div class='attachment'>Attachment: {}</div>",
                            att_name
                        ));
                    }
                }
                html_content.push_str("</div>");
                count += 1;
            }

            let _ = window.emit(
                "export_progress",
                ExportProgress {
                    current: i + 1,
                    total: options.channel_ids.len(),
                    channel_id: channel_id.to_string(),
                    status: "processing".to_string(),
                    processed_count: count,
                },
            );
        }
        html_content.push_str("</body></html>");

        let file_path = output_dir.join(format!("{}.html", channel_id));
        let mut file = File::create(file_path).map_err(|e| {
            AppError::new(&format!("Failed to create HTML file: {}", e), "io_error")
        })?;
        file.write_all(html_content.as_bytes())
            .map_err(|e| AppError::new(&format!("Failed to write HTML: {}", e), "io_error"))?;
    }

    op_manager.state.reset();
    let _ = window.emit("export_complete", ());
    Ok(())
}

#[tauri::command]
pub async fn start_attachment_harvest(
    app_handle: AppHandle,
    window: Window,
    options: ExportOptions,
) -> Result<(), AppError> {
    let identity = Vault::get_active_identity(&app_handle)?;
    let token = identity.token;
    let is_bearer = identity.is_oauth;
    let current_user_id = identity.id;

    // Use options.format to suppress warning
    if options.format != "html" && options.format != "raw" {
        Logger::warn(
            &app_handle,
            "[HARVEST] Unknown export format requested, proceeding with default.",
            None,
        );
    }

    let api_handle = app_handle.state::<ApiHandle>();
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.prepare();
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    let output_dir = PathBuf::from(&options.output_path);
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).map_err(|e| {
            AppError::new(&format!("Failed to create output dir: {}", e), "io_error")
        })?;
    }

    let mut total_processed = 0;

    for (i, channel_id) in options.channel_ids.iter().enumerate() {
        op_manager.state.wait_if_paused().await;
        if op_manager.state.should_abort.load(Ordering::SeqCst) {
            break;
        }

        let _ = window.emit(
            "export_progress",
            ExportProgress {
                current: i + 1,
                total: options.channel_ids.len(),
                channel_id: channel_id.clone(),
                status: "fetching_messages".to_string(),
                processed_count: total_processed,
            },
        );

        let mut last_id: Option<String> = None;
        loop {
            op_manager.state.wait_if_paused().await;
            if op_manager.state.should_abort.load(Ordering::SeqCst) {
                break;
            }

            let mut url = format!(
                "https://discord.com/api/v9/channels/{}/messages?limit=100",
                channel_id
            );
            if let Some(id) = &last_id {
                url.push_str(&format!("&before={}", id));
            }

            let res_content = api_handle
                .send_request(reqwest::Method::GET, &url, None, &token, is_bearer, false)
                .await;
            let messages: Vec<serde_json::Value> = match res_content {
                Ok(ApiResponseContent::Json(json)) => {
                    serde_json::from_value(json).map_err(AppError::from)?
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
                let author_id = msg["author"]["id"].as_str().unwrap_or_default();
                let is_own = author_id == current_user_id;

                let should_process = match options.direction.as_str() {
                    "sent" => is_own,
                    "received" => !is_own,
                    _ => true,
                };

                if !should_process {
                    continue;
                }

                if let Some(attachments) = msg["attachments"].as_array() {
                    for att in attachments {
                        let file_url = att["url"].as_str().unwrap_or_default();
                        let filename = att["filename"].as_str().unwrap_or("unknown_file");
                        let att_id = att["id"].as_str().unwrap_or("0");

                        let safe_filename = format!("{}_{}", att_id, filename);
                        let save_path = output_dir.join(safe_filename);

                        if let Err(e) =
                            download_file(&api_handle, file_url, &save_path, &token, is_bearer)
                                .await
                        {
                            Logger::error(
                                &app_handle,
                                &format!("Failed to download attachment: {}", e.user_message),
                                None,
                            );
                        } else {
                            total_processed += 1;
                        }
                    }
                }
            }
        }
    }

    op_manager.state.reset();
    let _ = window.emit("export_complete", ());
    Ok(())
}

#[tauri::command]
pub async fn start_guild_user_archive(
    app_handle: AppHandle,
    window: Window,
    guild_id: String,
    output_path: String,
    _options: ExportOptions,
) -> Result<(), AppError> {
    let identity = Vault::get_active_identity(&app_handle)?;
    let token = identity.token;
    let is_bearer = identity.is_oauth;
    let current_user_id = identity.id;

    let api_handle = app_handle.state::<ApiHandle>();
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.prepare();
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    let channels_res_content = api_handle
        .send_request(
            reqwest::Method::GET,
            &format!("https://discord.com/api/v9/guilds/{}/channels", guild_id),
            None,
            &token,
            is_bearer,
            false,
        )
        .await?;

    let channels: Vec<serde_json::Value> = match channels_res_content {
        ApiResponseContent::Json(v) => serde_json::from_value(v).map_err(AppError::from)?,
        _ => {
            return Err(AppError::new(
                "Expected JSON, received raw bytes",
                "unexpected_response_type",
            ));
        }
    };

    let output_file = File::create(&output_path)
        .map_err(|e| AppError::new(&format!("Failed to create zip: {}", e), "io_error"))?;
    let mut zip = ZipWriter::new(output_file);
    let zip_options =
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut log_accumulator = String::new();

    for (i, channel) in channels.iter().enumerate() {
        let channel_id = channel["id"].as_str().unwrap_or_default();
        let channel_name = channel["name"].as_str().unwrap_or("unknown");
        let c_type = channel["type"].as_u64().unwrap_or(0);

        if c_type != 0 && c_type != 2 {
            continue;
        }

        let _ = window.emit(
            "export_progress",
            ExportProgress {
                current: i + 1,
                total: channels.len(),
                channel_id: channel_id.to_string(),
                status: "archiving_channel".to_string(),
                processed_count: 0,
            },
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

            let res_content = api_handle
                .send_request(reqwest::Method::GET, &url, None, &token, is_bearer, false)
                .await;
            let messages: Vec<serde_json::Value> = match res_content {
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
                if msg["author"]["id"].as_str().unwrap_or_default() == current_user_id {
                    let ts = msg["timestamp"].as_str().unwrap_or("");
                    let content = msg["content"].as_str().unwrap_or("");
                    log_accumulator.push_str(&format!(
                        "[{}] [#{}]: {}
",
                        ts, channel_name, content
                    ));
                }
            }
        }
    }

    zip.start_file("my_messages.txt", zip_options)
        .map_err(|e| AppError::new(&format!("Zip error: {}", e), "io_error"))?;
    zip.write_all(log_accumulator.as_bytes())
        .map_err(|e| AppError::new(&format!("Zip write error: {}", e), "io_error"))?;
    zip.finish()
        .map_err(|e| AppError::new(&format!("Zip finish error: {}", e), "io_error"))?;

    op_manager.state.reset();
    let _ = window.emit("export_complete", ());
    Ok(())
}
