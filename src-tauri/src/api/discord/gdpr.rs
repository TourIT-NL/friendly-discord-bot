// src-tauri/src/api/discord/gdpr.rs

use crate::core::error::AppError;
use crate::core::logger::Logger;
use std::fs::File;
use std::io::Read;
use tauri::AppHandle;
use zip::ZipArchive;

#[derive(serde::Serialize)]
pub struct GdprDiscovery {
    pub channel_ids: Vec<String>,
    pub guild_ids: Vec<String>,
    pub user_ids: Vec<String>,
}

#[tauri::command]
pub async fn process_gdpr_data(
    app_handle: AppHandle,
    zip_path: String,
) -> Result<GdprDiscovery, AppError> {
    Logger::info(
        &app_handle,
        &format!("[GDPR] Parsing data package: {}", zip_path),
        None,
    );

    let file = File::open(&zip_path)
        .map_err(|e| AppError::new(&format!("Failed to open data package: {}", e), "io_error"))?;

    let mut archive = ZipArchive::new(file)
        .map_err(|e| AppError::new(&format!("Invalid zip archive: {}", e), "zip_error"))?;

    let mut channel_ids = std::collections::HashSet::new();
    let mut guild_ids = std::collections::HashSet::new();
    let mut user_ids = std::collections::HashSet::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let name = file.name().to_string();

        if name.starts_with("messages/c") && name.ends_with("/messages.csv") {
            // Extract channel ID from path: messages/c12345/messages.csv
            if let Some(id) = name.split('/').nth(1).and_then(|s| s.get(1..)) {
                channel_ids.insert(id.to_string());
            }
        } else if name == "servers/index.json" {
            let mut content = String::new();
            let _ = file.read_to_string(&mut content);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content)
                && let Some(arr) = json.as_array()
            {
                for server in arr {
                    if let Some(id) = server["id"].as_str() {
                        guild_ids.insert(id.to_string());
                    }
                }
            }
        } else if name == "account/user.json" {
            // Maybe find friends? Actually relationships/index.json is better
        } else if name == "relationships/index.json" {
            let mut content = String::new();
            let _ = file.read_to_string(&mut content);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content)
                && let Some(arr) = json.as_array()
            {
                for rel in arr {
                    if let Some(id) = rel["id"].as_str() {
                        user_ids.insert(id.to_string());
                    }
                }
            }
        }
    }

    Logger::info(
        &app_handle,
        &format!(
            "[GDPR] Discovery complete: {} channels, {} guilds, {} relationships",
            channel_ids.len(),
            guild_ids.len(),
            user_ids.len()
        ),
        None,
    );

    Ok(GdprDiscovery {
        channel_ids: channel_ids.into_iter().collect(),
        guild_ids: guild_ids.into_iter().collect(),
        user_ids: user_ids.into_iter().collect(),
    })
}
