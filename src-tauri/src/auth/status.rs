// src-tauri/src/auth/status.rs

use super::types::DiscordStatus;
use crate::core::error::AppError;
use crate::core::forensics::auditor::SessionAuditor;
use crate::core::logger::Logger;
use std::time::Duration;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

#[tauri::command]
pub async fn check_discord_status(app_handle: tauri::AppHandle) -> Result<DiscordStatus, AppError> {
    let mut s = System::new();
    s.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::default());

    let discord_proc = s.processes().values().find(|p| {
        let name = p.name().to_string_lossy().to_ascii_lowercase();
        // Check for common Discord executable names
        name.contains("discord") || name.contains("discordptb") || name.contains("discordcanary")
    });

    let is_running = discord_proc.is_some();
    let active_pid = discord_proc.map(|p| p.pid().as_u32());

    if is_running {
        Logger::trace(
            &app_handle,
            &format!("[STATUS] Discord process found: PID {:?}", active_pid),
            None,
        );
    }

    // Elaborate installation path detection
    let installation_path =
        discord_proc.and_then(|p| p.exe().map(|path| path.to_string_lossy().to_string()));

    let mut detected_port = None;
    let rpc_available = (6463..=6472).any(|port| {
        let addr = format!("127.0.0.1:{}", port);
        // Connect with timeout to check for RPC availability
        match std::net::TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            Duration::from_millis(150),
        ) {
            Ok(_) => {
                detected_port = Some(port);
                true
            }
            Err(_) => false,
        }
    });

    if rpc_available {
        Logger::trace(
            &app_handle,
            &format!(
                "[STATUS] Discord RPC gateway detected on port {}",
                detected_port.unwrap()
            ),
            None,
        );
    }

    let browser_detected = s.processes().values().any(|p| {
        let n = p.name().to_string_lossy().to_ascii_lowercase();
        ["chrome", "firefox", "msedge", "brave", "safari", "opera"]
            .iter()
            .any(|b| n.contains(b))
    });

    // Only extrapolate if we don't have it in vault
    if crate::core::vault::Vault::get_credential(&app_handle, "client_id").is_err() {
        // This is safe to run as it will only return early if it finds one.
        // But let's avoid it if we already have it.
        let _ = SessionAuditor::extrapolate_client_id(&app_handle);
    }

    Ok(DiscordStatus {
        is_running,
        rpc_available,
        browser_detected,
        active_pid,
        detected_port,
        installation_path,
    })
}
