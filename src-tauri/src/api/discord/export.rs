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
    pub direction: String,
    #[serde(alias = "includeAttachments")]
    pub include_attachments: bool,
    #[serde(alias = "exportFormat")]
    pub format: String,
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
    let res = api_handle
        .send_request(
            reqwest::Method::GET,
            url,
            None,
            token,
            is_bearer,
            true,
            None,
            None,
            None,
            None,
        )
        .await?;
    match res {
        ApiResponseContent::Bytes(b) => {
            let mut f = File::create(save_path)?;
            f.write_all(&b)?;
            Ok(())
        }
        _ => Err(AppError::new("Expected bytes", "api_mismatch")),
    }
}

#[tauri::command]
pub async fn start_chat_html_export(
    app_handle: AppHandle,
    _window: Window,
    options: ExportOptions,
) -> Result<(), AppError> {
    let ident = Vault::get_active_identity(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.prepare();
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    Logger::info(
        &app_handle,
        &format!(
            "[OP] Exporting {} channels to {}",
            options.channel_ids.len(),
            options.format
        ),
        None,
    );

    let output_dir = PathBuf::from(&options.output_path);
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)?;
    }

    for channel_id in &options.channel_ids {
        let mut html = String::from("<html><head><title>Export</title></head><body>");
        let mut last_id: Option<String> = None;
        loop {
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
                    &ident.token,
                    ident.is_oauth,
                    false,
                    None,
                    None,
                    None,
                    None,
                )
                .await;
            let messages: Vec<serde_json::Value> = match res {
                Ok(ApiResponseContent::Json(v)) => serde_json::from_value(v)?,
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
                let is_own = author_id == ident.id;

                let should_export = match options.direction.as_str() {
                    "sent" => is_own,
                    "received" => !is_own,
                    _ => true,
                };

                if !should_export {
                    continue;
                }

                let author = msg["author"]["username"].as_str().unwrap_or("Unknown");
                let content = msg["content"].as_str().unwrap_or("");
                html.push_str(&format!("<p><b>{}</b>: {}</p>", author, content));

                if options.include_attachments {
                    if let Some(atts) = msg["attachments"].as_array() {
                        for att in atts {
                            html.push_str(&format!(
                                "<p><i>Attachment: {}</i></p>",
                                att["filename"].as_str().unwrap_or("file")
                            ));
                        }
                    }
                }
            }
        }
        html.push_str("</body></html>");
        let mut f = File::create(output_dir.join(format!("{}.html", channel_id)))?;
        f.write_all(html.as_bytes())?;
    }
    op_manager.state.reset();
    Ok(())
}

#[tauri::command]
pub async fn start_attachment_harvest(
    app_handle: AppHandle,
    _window: Window,
    options: ExportOptions,
) -> Result<(), AppError> {
    let ident = Vault::get_active_identity(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.prepare();
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    Logger::info(
        &app_handle,
        "[OP] Harvesting attachments from selected nodes",
        None,
    );

    let output_dir = PathBuf::from(&options.output_path);
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)?;
    }

    for channel_id in &options.channel_ids {
        let mut last_id: Option<String> = None;
        loop {
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
                    &ident.token,
                    ident.is_oauth,
                    false,
                    None,
                    None,
                    None,
                    None,
                )
                .await;
            let messages: Vec<serde_json::Value> = match res {
                Ok(ApiResponseContent::Json(v)) => serde_json::from_value(v)?,
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
                let is_own = author_id == ident.id;

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
                        let filename = att["filename"].as_str().unwrap_or("file");
                        let save_path = output_dir.join(format!(
                            "{}_{}",
                            att["id"].as_str().unwrap_or("0"),
                            filename
                        ));
                        let _ = download_file(
                            &api_handle,
                            file_url,
                            &save_path,
                            &ident.token,
                            ident.is_oauth,
                        )
                        .await;
                    }
                }
            }
        }
    }
    op_manager.state.reset();
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
    let ident = Vault::get_active_identity(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();
    let op_manager = app_handle.state::<OperationManager>();
    op_manager.state.prepare();
    op_manager.state.is_running.store(true, Ordering::SeqCst);

    Logger::info(
        &app_handle,
        &format!("[OP] Archiving guild messages for node {}", guild_id),
        None,
    );

    let res = api_handle
        .send_request(
            reqwest::Method::GET,
            &format!("https://discord.com/api/v9/guilds/{}/channels", guild_id),
            None,
            &ident.token,
            ident.is_oauth,
            false,
            None,
            None,
            None,
            None,
        )
        .await?;
    let channels: Vec<serde_json::Value> = match res {
        ApiResponseContent::Json(v) => serde_json::from_value(v)?,
        _ => return Err(AppError::new("Expected JSON", "api_mismatch")),
    };

    let f = File::create(&output_path)?;
    let mut zip = ZipWriter::new(f);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for (i, ch) in channels.iter().enumerate() {
        let id = ch["id"].as_str().unwrap_or_default();
        let name = ch["name"].as_str().unwrap_or("channel");
        let ctype = ch["type"].as_u64().unwrap_or(0);
        if ctype != 0 && ctype != 2 {
            continue;
        }

        let _ = window.emit(
            "export_progress",
            ExportProgress {
                current: i + 1,
                total: channels.len(),
                channel_id: id.to_string(),
                status: "archiving".to_string(),
                processed_count: 0,
            },
        );

        let mut log = String::new();
        let mut last_id: Option<String> = None;
        loop {
            let mut url = format!(
                "https://discord.com/api/v9/channels/{}/messages?limit=100",
                id
            );
            if let Some(lid) = &last_id {
                url.push_str(&format!("&before={}", lid));
            }

            let res = api_handle
                .send_request(
                    reqwest::Method::GET,
                    &url,
                    None,
                    &ident.token,
                    ident.is_oauth,
                    false,
                    None,
                    None,
                    None,
                    None,
                )
                .await;
            let messages: Vec<serde_json::Value> = match res {
                Ok(ApiResponseContent::Json(v)) => serde_json::from_value(v)?,
                _ => break,
            };
            if messages.is_empty() {
                break;
            }
            last_id = messages
                .last()
                .and_then(|m| m["id"].as_str().map(|s| s.to_string()));

            for msg in messages {
                if msg["author"]["id"].as_str() == Some(&ident.id) {
                    log.push_str(&format!(
                        "[{}] {}: {}\n",
                        msg["timestamp"].as_str().unwrap_or(""),
                        name,
                        msg["content"].as_str().unwrap_or("")
                    ));
                }
            }
        }
        zip.start_file(format!("{}.txt", name), opts)
            .map_err(|e| AppError::new(&e.to_string(), "zip_error"))?;
        zip.write_all(log.as_bytes())?;
    }
    zip.finish()
        .map_err(|e| AppError::new(&e.to_string(), "zip_error"))?;
    op_manager.state.reset();
    Ok(())
}
