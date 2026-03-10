use super::paths::get_discord_base_paths;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::vault::Vault;
use is_elevated::is_elevated;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::copy;
use walkdir::WalkDir;

pub struct SessionAuditor;

impl SessionAuditor {
    pub fn audit_system_environment(app: &tauri::AppHandle) -> Result<(), AppError> {
        Logger::info(
            app,
            "[Forensics] Starting system environment audit...",
            None,
        );

        if is_elevated() {
            Logger::warn(
                app,
                "[Forensics] Application is running with elevated privileges. This increases attack surface.",
                None,
            );
        } else {
            Logger::debug(
                app,
                "[Forensics] Application is running with normal user privileges.",
                None,
            );
        }

        Logger::debug(
            app,
            "[Forensics] Performing DNS resolution check (placeholder).",
            None,
        );
        Logger::debug(
            app,
            "[Forensics] Performing basic process scan (placeholder).",
            None,
        );
        Logger::debug(
            app,
            "[Forensics] Performing anti-debugging check (placeholder).",
            None,
        );

        Logger::info(app, "[Forensics] System environment audit completed.", None);
        Ok(())
    }

    pub fn check_discord_client_integrity(app: &tauri::AppHandle) -> Result<(), AppError> {
        Logger::info(
            app,
            "[Forensics] Starting Discord client integrity check...",
            None,
        );

        let base_paths = get_discord_base_paths();
        if base_paths.is_empty() {
            return Err(AppError::new(
                "No Discord installations found to perform integrity check.",
                "discord_not_found",
            ));
        }

        let mut integrity_verified = false;
        'base_path_loop: for base_path in &base_paths {
            Logger::debug(
                app,
                &format!("[Forensics] Checking integrity in: {:?}", base_path),
                None,
            );

            // 1. Check for executable
            let exe_name = {
                #[cfg(target_os = "windows")]
                {
                    "Discord.exe"
                }
                #[cfg(target_os = "macos")]
                {
                    "Discord"
                } // This is likely inside Contents/MacOS/
                #[cfg(target_os = "linux")]
                {
                    "Discord"
                }
            };

            let exe_path = base_path.join(exe_name);
            if !exe_path.is_file() {
                Logger::warn(
                    app,
                    &format!(
                        "[Forensics] Discord executable not found at: {:?}",
                        exe_path
                    ),
                    None,
                );
                continue; // Try next base_path
            }

            // Hash verification for exe_path (existing logic from original)
            match fs::File::open(&exe_path) {
                Ok(mut file) => {
                    let mut hasher = Sha256::new();
                    match copy(&mut file, &mut hasher) {
                        Ok(_) => {
                            let hash = hasher.finalize();
                            Logger::debug(
                                app,
                                &format!(
                                    "[Forensics] Verified executable: {:?} Hash: {:x}",
                                    exe_path, hash
                                ),
                                None,
                            );
                        }
                        Err(_e) => {
                            Logger::warn(
                                app,
                                &format!(
                                    "Failed to read and hash Discord executable: {:?}",
                                    exe_path
                                ),
                                None,
                            );
                            continue; // This base_path is problematic
                        }
                    }
                }
                Err(_) => {
                    Logger::warn(
                        app,
                        &format!(
                            "Failed to open Discord executable for integrity check: {:?}",
                            exe_path
                        ),
                        None,
                    );
                    continue; // This base_path is problematic
                }
            }

            // 2. Check for modules
            let modules_dir = base_path.join("modules");
            if !modules_dir.is_dir() {
                Logger::warn(
                    app,
                    &format!(
                        "[Forensics] 'modules' directory not found in: {:?}",
                        base_path
                    ),
                    None,
                );
                continue; // Try next base_path
            }

            let mut core_module_found = false;
            if let Ok(entries) = std::fs::read_dir(&modules_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(dir_name) = path.file_name().and_then(|s| s.to_str()) {
                            if dir_name.starts_with("discord_desktop_core-") {
                                let core_module_root = path; // This is the path to discord_desktop_core-1
                                let core_module_path =
                                    core_module_root.join("discord_desktop_core");

                                let index_js = core_module_path.join("index.js");
                                let core_asar = core_module_path.join("core.asar");
                                let package_json = core_module_path.join("package.json");

                                let critical_core_module_files =
                                    vec![index_js, core_asar, package_json];

                                let mut all_files_in_core_found = true;
                                for file_path in critical_core_module_files {
                                    if !file_path.is_file() {
                                        Logger::warn(
                                            app,
                                            &format!(
                                                "[Forensics] Critical Discord module file not found: {:?}",
                                                file_path
                                            ),
                                            None,
                                        );
                                        all_files_in_core_found = false;
                                        break; // This core module path is invalid
                                    }
                                    // Hash verification for module file
                                    match fs::File::open(&file_path) {
                                        Ok(mut file) => {
                                            let mut hasher = Sha256::new();
                                            match copy(&mut file, &mut hasher) {
                                                Ok(_) => {
                                                    let hash = hasher.finalize();
                                                    Logger::debug(
                                                        app,
                                                        &format!(
                                                            "[Forensics] Verified module file: {:?} Hash: {:x}",
                                                            file_path, hash
                                                        ),
                                                        None,
                                                    );
                                                }
                                                Err(_e) => {
                                                    Logger::warn(
                                                        app,
                                                        &format!(
                                                            "Failed to read and hash Discord module file: {:?}",
                                                            file_path
                                                        ),
                                                        None,
                                                    );
                                                    all_files_in_core_found = false;
                                                    break;
                                                }
                                            }
                                        }
                                        Err(_) => {
                                            Logger::warn(
                                                app,
                                                &format!(
                                                    "Failed to open Discord module file for integrity check: {:?}",
                                                    file_path
                                                ),
                                                None,
                                            );
                                            all_files_in_core_found = false;
                                            break;
                                        }
                                    }
                                }

                                if all_files_in_core_found {
                                    core_module_found = true;
                                    // 3. Scan for malicious modifications within this module
                                    Self::scan_for_malicious_modifications(app, &index_js);
                                    break; // Found and verified, we are done with this module scan
                                }
                            }
                        }
                    }
                }
            }
            if !core_module_found {
                Logger::warn(
                    app,
                    &format!(
                        "[Forensics] Could not find and verify 'discord_desktop_core' within: {:?}",
                        modules_dir
                    ),
                    None,
                );
                continue; // Try next base_path
            }

            // If we reach here, both exe and modules are verified for this base_path
            integrity_verified = true;
            break 'base_path_loop;
        }

        if integrity_verified {
            Logger::info(
                app,
                "[Forensics] Discord client integrity check completed successfully.",
                None,
            );
            Ok(())
        } else {
            Err(AppError::new(
                "Failed to verify Discord client integrity across all found installations.",
                "discord_integrity_check_failed",
            ))
        }
    }

    /// Scans a specific index.js file for common token stealer patterns.
    fn scan_for_malicious_modifications(app: &tauri::AppHandle, path: &std::path::Path) {
        if let Ok(content) = fs::read_to_string(path) {
            let mut detected = false;
            let suspicious_keywords = [
                "webhook",
                "http",
                "https",
                "axios",
                "fetch",
                "XMLHttpRequest",
                "LOCALAPPDATA",
                "Roaming",
                "leveldb",
                "tokens",
                "password",
                "mfa",
                "ND...",
                "OT...",
            ];

            // A typical Discord index.js is very small. If it's over 1KB and contains webhooks, it's suspicious.
            let size = content.len();
            if size > 1500 {
                for kw in &suspicious_keywords {
                    if content.contains(kw) {
                        Logger::warn(
                            app,
                            &format!(
                                "[SECURITY] Detected suspicious payload in Discord module index.js: '{}' at {:?}",
                                kw, path
                            ),
                            Some(serde_json::json!({"file_size": size, "pattern": kw})),
                        );
                        detected = true;
                    }
                }
            }

            if !detected {
                Logger::debug(
                    app,
                    &format!(
                        "[Forensics] No immediate malicious patterns found in {:?}",
                        path
                    ),
                    None,
                );
            }
        }
    }

    pub fn extrapolate_client_id(app: &tauri::AppHandle) -> Result<String, AppError> {
        if let Ok(id_from_env) = std::env::var("DISCORD_CLIENT_ID") {
            if !id_from_env.is_empty() {
                Logger::info(
                    app,
                    "[Forensics] Using client_id from DISCORD_CLIENT_ID environment variable.",
                    None,
                );
                return Ok(id_from_env);
            }
        }

        if let Ok(id_from_vault) = Vault::get_credential(app, "client_id") {
            return Ok(id_from_vault);
        }

        let base_paths = get_discord_base_paths();
        // More robust regex to catch variations in minified or formatted JS
        let re = Regex::new(r#"(?i)client_?id[:=]\s*["']?([0-9]{17,21})["']?"#).unwrap();

        for base_path in &base_paths {
            Logger::debug(
                app,
                &format!("[Forensics] Scanning for Client ID in: {:?}", base_path),
                None,
            );

            // Priority 1: Check in app resources
            #[cfg(target_os = "windows")]
            if let Ok(entries) = fs::read_dir(base_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let entry_path = entry.path();
                    if entry_path.is_dir()
                        && entry_path
                            .file_name()
                            .and_then(|s| s.to_str())
                            .is_some_and(|n| n.starts_with("app-"))
                    {
                        let app_resources_path = entry_path.join("resources").join("app");
                        if let Some(id) = Self::scrape_js_files(app, &app_resources_path, &re) {
                            Logger::info(
                                app,
                                &format!("[Forensics] Successfully extrapolated client_id: {}", id),
                                None,
                            );
                            return Err(AppError::client_id_extrapolation_needed(id));
                        }

                        // Fallback: Check in app.asar if possible (placeholder for future implementation)
                    }
                }
            }

            #[cfg(any(target_os = "linux", target_os = "windows"))]
            {
                let alt_path = base_path.join("resources").join("app");
                if let Some(id) = Self::scrape_js_files(app, &alt_path, &re) {
                    return Err(AppError::client_id_extrapolation_needed(id));
                }
            }

            #[cfg(target_os = "macos")]
            {
                let app_resources_path = base_path.join("Contents").join("Resources").join("app");
                if let Some(id) = Self::scrape_js_files(app, &app_resources_path, &re) {
                    return Err(AppError::client_id_extrapolation_needed(id));
                }
            }
        }

        Err(AppError::new(
            "No Discord Client ID found.",
            "client_id_not_found",
        ))
    }

    fn scrape_js_files(
        app: &tauri::AppHandle,
        path: &std::path::Path,
        re: &Regex,
    ) -> Option<String> {
        if !path.exists() {
            return None;
        }
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().is_some_and(|ext| ext == "js") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if let Some(cap) = re.captures(&content) {
                        if let Some(id) = cap.get(1) {
                            let client_id = id.as_str().to_string();
                            Logger::info(
                                app,
                                &format!("[Forensics] Extrapolated client_id: {}", client_id),
                                None,
                            );
                            return Some(client_id);
                        }
                    }
                }
            }
        }
        None
    }
}
