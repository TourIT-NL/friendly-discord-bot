// src-tauri/src/core/forensics/metadata.rs

use crate::core::error::AppError;
use crate::core::logger::Logger;
use little_exif::metadata::Metadata;
use std::path::Path;
use tauri::AppHandle;

/// Forensic-level media metadata sanitizer.
pub struct MetadataStripper;

impl MetadataStripper {
    /// Strips all EXIF/IPTC/XMP metadata from a target image file.
    /// Supports JPEG, PNG, and WebP formats.
    pub fn strip_file(app: &AppHandle, file_path: &Path) -> Result<(), AppError> {
        Logger::debug(
            app,
            &format!("[FORENSICS] Stripping metadata from {:?}", file_path),
            None,
        );

        // 1. Load metadata from file
        let _metadata = Metadata::new_from_path(file_path).map_err(|e| AppError {
            user_message: "Failed to read image metadata.".into(),
            technical_details: Some(e.to_string()),
            ..Default::default()
        })?;

        // 2. Clear all metadata entries
        // little_exif doesn't have a direct 'clear_all', so we re-initialize an empty metadata object
        // and write it back to the same path.
        let empty_metadata = Metadata::new();

        empty_metadata
            .write_to_file(file_path)
            .map_err(|e| AppError {
                user_message: "Failed to sanitize image metadata.".into(),
                technical_details: Some(e.to_string()),
                ..Default::default()
            })?;

        Logger::info(
            app,
            &format!("[FORENSICS] Media sanitized: {:?}", file_path),
            None,
        );
        Ok(())
    }
}
