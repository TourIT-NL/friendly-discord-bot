// src-tauri/src/api/discord/export.rs

use crate::api::rate_limiter::ApiHandle;
use crate::api::rate_limiter::handle::RequestConfig;
use crate::core::error::AppError;
use crate::core::op_manager::OperationManager;
use crate::core::vault::Vault;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager, Window};
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

#[allow(dead_code)]
#[derive(serde::Deserialize, Clone)]
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

#[allow(dead_code)]
async fn download_file(
    api_handle: &ApiHandle,
    url: &str,
    save_path: &Path,
    token: &str,
    is_bearer: bool,
) -> Result<(), AppError> {
    let response_content = api_handle
        .send_request(
            reqwest::Method::GET,
            url,
            None,
            token,
            is_bearer,
            RequestConfig {
                return_raw_bytes: true,
                ..Default::default()
            },
        )
        .await?;

    match response_content {
        crate::api::rate_limiter::types::ApiResponseContent::Bytes(bytes) => {
            let mut file = File::create(save_path).map_err(|e| AppError {
                user_message: format!("Failed to create file: {}", e),
                ..Default::default()
            })?;
            file.write_all(&bytes).map_err(|e| AppError {
                user_message: format!("Failed to write file: {}", e),
                ..Default::default()
            })?;
            Ok(())
        }
        _ => Err(AppError {
            user_message: "Expected raw bytes, received JSON".into(),
            ..Default::default()
        }),
    }
}

#[tauri::command]
pub async fn start_chat_html_export(
    app_handle: AppHandle,
    window: Window,
    options: ExportOptions,
) -> Result<(), AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.prepare();
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    let output_dir = PathBuf::from(&options.output_path);
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).map_err(|e| AppError {
            user_message: format!("Failed to create output dir: {}", e),
            ..Default::default()
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

        let mut html_content = String::from(
            "<html><head><style>
            body { font-family: sans-serif; background: #313338; color: #dbdee1; padding: 20px; }
            .message { margin-bottom: 15px; padding: 10px; border-radius: 8px; background: #2b2d31; }
            .author { font-weight: bold; color: #f2f3f5; margin-right: 10px; }
            .timestamp { font-size: 0.8em; color: #949ba4; }
            .content { margin-top: 5px; white-space: pre-wrap; }
            .attachment { margin-top: 10px; padding: 5px; border: 1px solid #4e5058; border-radius: 4px; display: inline-block; }
        </style></head><body>",
        );

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

            for msg in messages {
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
        let mut file = File::create(file_path).map_err(|e| AppError {
            user_message: format!("Failed to create HTML file: {}", e),
            ..Default::default()
        })?;
        file.write_all(html_content.as_bytes())
            .map_err(|e| AppError {
                user_message: format!("Failed to write HTML: {}", e),
                ..Default::default()
            })?;
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
    start_chat_html_export(app_handle, window, options).await
}

#[tauri::command]
pub async fn start_guild_user_archive(
    app_handle: AppHandle,
    window: Window,
    guild_id: String,
    output_path: String,
) -> Result<(), AppError> {
    let identity = Vault::get_active_identity(&app_handle)?;
    let token = identity.token;
    let is_bearer = identity.is_oauth;
    let current_user_id = identity.id;

    let api_handle = app_handle.state::<ApiHandle>();
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.prepare();
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    let json = api_handle
        .send_request_json(
            reqwest::Method::GET,
            &format!("https://discord.com/api/v9/guilds/{}/channels", guild_id),
            None,
            &token,
            is_bearer,
            None,
        )
        .await?;

    let channels: Vec<serde_json::Value> = serde_json::from_value(json).map_err(AppError::from)?;

    let output_file = File::create(&output_path).map_err(|e| AppError {
        user_message: format!("Failed to create zip: {}", e),
        ..Default::default()
    })?;
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

            for msg in messages {
                if msg["author"]["id"].as_str() == Some(&current_user_id) {
                    let ts = msg["timestamp"].as_str().unwrap_or("");
                    let content = msg["content"].as_str().unwrap_or("");
                    log_accumulator
                        .push_str(&format!("[{}] [#{}]: {}\n", ts, channel_name, content));
                }
            }
        }
    }

    zip.start_file("my_messages.txt", zip_options)
        .map_err(|e| AppError {
            user_message: format!("Zip error: {}", e),
            ..Default::default()
        })?;
    zip.write_all(log_accumulator.as_bytes())
        .map_err(|e| AppError {
            user_message: format!("Zip write error: {}", e),
            ..Default::default()
        })?;
    zip.finish().map_err(|e| AppError {
        user_message: format!("Zip finish error: {}", e),
        ..Default::default()
    })?;

    op_manager.state.reset();
    let _ = window.emit("export_complete", ());
    Ok(())
}
