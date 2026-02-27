// src-tauri/src/core/forensics/burner.rs

use crate::core::cache::CacheManager;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::vault::Vault;
use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use tauri::{AppHandle, Manager};

/// Anti-forensic data destruction module.
/// Implements secure file shredding to ensure no traces remain on the physical storage medium.
pub struct BurnerManager;

impl BurnerManager {
    /// Overwrites a file multiple times with different patterns before deletion.
    /// This is designed to defeat forensic recovery tools.
    fn shred_file(path: &Path) -> Result<(), AppError> {
        if !path.exists() {
            return Ok(());
        }

        let file_size = fs::metadata(path)?.len();
        let mut file = OpenOptions::new().write(true).open(path)?;

        let mut buffer = vec![0u8; 65536]; // 64KB chunks

        // Pass 1: Zeroize (All bits to 0)
        file.seek(SeekFrom::Start(0))?;
        let mut written = 0;
        while written < file_size {
            let to_write = std::cmp::min(buffer.len() as u64, file_size - written);
            file.write_all(&buffer[..to_write as usize])?;
            written += to_write;
        }
        file.sync_all()?;

        // Pass 2: Saturate (All bits to 1)
        file.seek(SeekFrom::Start(0))?;
        buffer.fill(0xFF);
        written = 0;
        while written < file_size {
            let to_write = std::cmp::min(buffer.len() as u64, file_size - written);
            file.write_all(&buffer[..to_write as usize])?;
            written += to_write;
        }
        file.sync_all()?;

        // Pass 3: Randomize (Entropy injection)
        file.seek(SeekFrom::Start(0))?;
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        written = 0;
        while written < file_size {
            let to_write = std::cmp::min(buffer.len() as u64, file_size - written);
            rng.fill_bytes(&mut buffer[..to_write as usize]);
            file.write_all(&buffer[..to_write as usize])?;
            written += to_write;
        }
        file.sync_all()?;

        // Finalize: Truncate and Remove
        file.set_len(0)?;
        drop(file);
        fs::remove_file(path)?;

        Ok(())
    }

    /// Initiates the Burner Protocol: Shreds all application data files.
    pub fn initiate_burner_protocol(app: &AppHandle) -> Result<(), AppError> {
        Logger::warn(
            app,
            "[BURNER] INITIATING ANTI-FORENSIC DATA DESTRUCTION...",
            None,
        );

        // 1. Shred Cache
        if let Ok(cache_path) = CacheManager::get_db_path(app) {
            let _ = Self::shred_file(&cache_path);
            // Also shred journal/WAL files if they exist
            let _ = Self::shred_file(&cache_path.with_extension("db-journal"));
            let _ = Self::shred_file(&cache_path.with_extension("db-wal"));
            let _ = Self::shred_file(&cache_path.with_extension("db-shm"));
        }

        // 2. Shred Vault & Identities (Local Fallbacks)
        let app_dir = app.path().app_local_data_dir().unwrap();
        if let Ok(entries) = fs::read_dir(&app_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if let Some(ext) = path.extension()
                    && ext == "secure"
                {
                    let _ = Self::shred_file(&path);
                }
            }
        }

        // 3. Clear Keyring (Non-destructive to bits, but clears logic)
        let _ = Vault::clear_all_data(app);

        // 4. Shred Logs
        let log_path = app_dir.join("app.log");
        let _ = Self::shred_file(&log_path);

        Logger::info(
            app,
            "[BURNER] BURNER PROTOCOL COMPLETE. SYSTEM IS CLEAN.",
            None,
        );
        Ok(())
    }
}
