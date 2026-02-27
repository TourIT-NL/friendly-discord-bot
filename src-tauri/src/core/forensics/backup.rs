// src-tauri/src/core/forensics/backup.rs

use crate::core::crypto::Crypto;
use crate::core::error::AppError;
use std::fs::File;
use std::io::Write;
use tauri::AppHandle;

#[allow(dead_code)]
pub struct ForensicBackup;

impl ForensicBackup {
    /// Creates an encrypted backup of messages from the local cache.
    #[allow(dead_code)]
    pub fn create_encrypted_backup(
        app: &AppHandle,
        channel_id: &str,
        output_path: &str,
    ) -> Result<(), AppError> {
        let conn = crate::core::cache::CacheManager::get_connection(app)?;
        let mut stmt = conn
            .prepare("SELECT content, author_id, timestamp FROM messages WHERE channel_id = ?1")
            .map_err(AppError::from)?;

        let messages: Vec<serde_json::Value> = stmt
            .query_map([channel_id], |row| {
                Ok(serde_json::json!({
                    "content": row.get::<_, String>(0)?,
                    "author_id": row.get::<_, String>(1)?,
                    "timestamp": row.get::<_, i64>(2)?,
                }))
            })
            .map_err(AppError::from)?
            .filter_map(|m| m.ok())
            .collect();

        let json = serde_json::to_string(&messages)?;
        let enc_key =
            crate::core::vault::encryption::EncryptionManager::get_or_create_encryption_key(app)?;
        let encrypted = Crypto::encrypt(&enc_key, &json)?;

        let mut file = File::create(output_path).map_err(AppError::from)?;
        file.write_all(encrypted.as_bytes())
            .map_err(AppError::from)?;

        Ok(())
    }
}
