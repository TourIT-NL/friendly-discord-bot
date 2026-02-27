// src-tauri/src/api/discord/footprint.rs

use crate::core::cache::CacheManager;
use crate::core::error::AppError;
use serde::Serialize;
use tauri::AppHandle;

#[derive(Serialize)]
pub struct FootprintStats {
    pub total_messages: i64,
    pub total_attachments: i64,
    pub server_densities: Vec<ServerDensity>,
}

#[derive(Serialize)]
pub struct ServerDensity {
    pub guild_id: String,
    pub guild_name: String,
    pub message_count: i64,
}

#[tauri::command]
pub async fn get_digital_footprint(app_handle: AppHandle) -> Result<FootprintStats, AppError> {
    let conn = CacheManager::get_connection(&app_handle)?;

    // 1. Total counts
    let total_messages: i64 = conn
        .query_row("SELECT COUNT(*) FROM messages", [], |r| r.get(0))
        .unwrap_or(0);
    let total_attachments: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM messages WHERE has_attachments = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    // 2. Density by server
    let mut stmt = conn
        .prepare(
            "
        SELECT g.id, g.name, COUNT(m.id) 
        FROM guilds g
        JOIN channels c ON g.id = c.guild_id
        JOIN messages m ON c.id = m.channel_id
        GROUP BY g.id
        ORDER BY COUNT(m.id) DESC
    ",
        )
        .map_err(AppError::from)?;

    let densities = stmt
        .query_map([], |row| {
            Ok(ServerDensity {
                guild_id: row.get(0)?,
                guild_name: row.get(1)?,
                message_count: row.get(2)?,
            })
        })
        .map_err(AppError::from)?;

    let mut server_densities = Vec::new();
    for d in densities {
        server_densities.push(d?);
    }

    Ok(FootprintStats {
        total_messages,
        total_attachments,
        server_densities,
    })
}
