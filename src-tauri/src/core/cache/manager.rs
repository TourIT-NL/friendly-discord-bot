// src-tauri/src/core/cache/manager.rs

use crate::core::cache::schema::SCHEMA;
use crate::core::crypto::Crypto;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use rusqlite::{Connection, params};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub struct CacheManager;

impl CacheManager {
    pub fn get_db_path(app: &AppHandle) -> Result<PathBuf, AppError> {
        let app_dir = app.path().app_local_data_dir().map_err(|e| AppError {
            user_message: "Failed to resolve app data directory.".into(),
            technical_details: Some(e.to_string()),
            ..Default::default()
        })?;
        Ok(app_dir.join("forensic_cache.db"))
    }

    pub fn get_connection(app: &AppHandle) -> Result<Connection, AppError> {
        let db_path = Self::get_db_path(app)?;
        let conn = Connection::open(db_path).map_err(|e| AppError {
            user_message: "Failed to initialize forensic cache.".into(),
            technical_details: Some(e.to_string()),
            ..Default::default()
        })?;

        // Initialize schema
        conn.execute_batch(SCHEMA).map_err(|e| AppError {
            user_message: "Cache schema initialization failed.".into(),
            technical_details: Some(e.to_string()),
            ..Default::default()
        })?;

        Ok(conn)
    }

    pub fn upsert_guilds(
        app: &AppHandle,
        identity_id: &str,
        guilds: &[crate::api::discord::types::Guild],
    ) -> Result<(), AppError> {
        let mut conn = Self::get_connection(app)?;
        let tx = conn.transaction().map_err(AppError::from)?;
        let now = chrono::Utc::now().timestamp();

        for guild in guilds {
            tx.execute(
                "INSERT OR REPLACE INTO guilds (id, identity_id, name, icon, owner, last_synced) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![guild.id, identity_id, guild.name, guild.icon, guild.owner, now],
            )?;
        }

        tx.commit().map_err(AppError::from)?;
        Logger::debug(
            app,
            &format!(
                "[CACHE] Indexed {} guilds for {}",
                guilds.len(),
                identity_id
            ),
            None,
        );
        Ok(())
    }

    pub fn upsert_channels(
        app: &AppHandle,
        identity_id: &str,
        guild_id: Option<&str>,
        channels: &[crate::api::discord::types::Channel],
    ) -> Result<(), AppError> {
        let mut conn = Self::get_connection(app)?;
        let tx = conn.transaction().map_err(AppError::from)?;
        let now = chrono::Utc::now().timestamp();

        for channel in channels {
            tx.execute(
                "INSERT OR REPLACE INTO channels (id, identity_id, guild_id, name, type, last_synced) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![channel.id, identity_id, guild_id, channel.name, channel.channel_type, now],
            )?;
        }

        tx.commit().map_err(AppError::from)?;
        Logger::debug(
            app,
            &format!(
                "[CACHE] Indexed {} channels for {}",
                channels.len(),
                identity_id
            ),
            None,
        );
        Ok(())
    }

    /// Optimized message insertion with forensic encryption.
    pub fn upsert_message(
        app: &AppHandle,
        identity_id: &str,
        msg: &serde_json::Value,
    ) -> Result<(), AppError> {
        let conn = Self::get_connection(app)?;
        let id = msg["id"].as_str().unwrap_or_default();
        let channel_id = msg["channel_id"].as_str().unwrap_or_default();
        let author_id = msg["author"]["id"].as_str().unwrap_or_default();
        let content = msg["content"].as_str().unwrap_or_default();
        let timestamp =
            chrono::DateTime::parse_from_rfc3339(msg["timestamp"].as_str().unwrap_or_default())
                .map(|dt| dt.timestamp_millis())
                .unwrap_or(0);
        let has_atts = !msg["attachments"]
            .as_array()
            .map(|a| a.is_empty())
            .unwrap_or(true);

        // Forensic Encryption Layer
        let enc_key =
            crate::core::vault::encryption::EncryptionManager::get_or_create_encryption_key(app)?;
        let encrypted_content = Crypto::encrypt(&enc_key, content)?;

        let _ = conn.execute(
            "INSERT OR REPLACE INTO messages (id, identity_id, channel_id, author_id, content, timestamp, has_attachments) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, identity_id, channel_id, author_id, encrypted_content, timestamp, has_atts],
        );
        Ok(())
    }

    pub fn search_messages(
        app: &AppHandle,
        query: &str,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let conn = Self::get_connection(app)?;
        let enc_key =
            crate::core::vault::encryption::EncryptionManager::get_or_create_encryption_key(app)?;

        let mut stmt = conn.prepare(
            "SELECT id, channel_id, identity_id, author_id, content, timestamp, has_attachments FROM messages"
        ).map_err(AppError::from)?;

        let message_rows = stmt
            .query_map([], |row| {
                let encrypted_content = row.get::<_, String>(4)?;
                // Decrypt for matching (this is O(N) unfortunately, but we're in local cache)
                let decrypted = Crypto::decrypt(&enc_key, &encrypted_content).unwrap_or_default();

                Ok((
                    serde_json::json!({
                        "id": row.get::<_, String>(0)?,
                        "channel_id": row.get::<_, String>(1)?,
                        "identity_id": row.get::<_, String>(2)?,
                        "author_id": row.get::<_, String>(3)?,
                        "content": decrypted.clone(),
                        "timestamp": row.get::<_, i64>(5)?,
                        "has_attachments": row.get::<_, bool>(6)?,
                    }),
                    decrypted,
                ))
            })
            .map_err(AppError::from)?;

        let mut results = Vec::new();
        for res in message_rows {
            if let Ok((json, content)) = res
                && content.to_lowercase().contains(&query.to_lowercase())
            {
                results.push(json);
            }
            if results.len() >= 1000 {
                break;
            }
        }
        Ok(results)
    }

    #[allow(dead_code)]
    pub fn wipe_cache(app: &AppHandle) -> Result<(), AppError> {
        let db_path = Self::get_db_path(app)?;
        if db_path.exists() {
            let _ = std::fs::remove_file(db_path);
            Logger::info(app, "[CACHE] Forensic cache wiped.", None);
        }
        Ok(())
    }
}
