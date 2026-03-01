// src-tauri/src/auth/status.rs

use super::types::DiscordStatus;
use crate::core::error::AppError;
use crate::core::forensics::auditor::SessionAuditor;
use std::time::Duration;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

#[tauri::command]
pub async fn check_discord_status(app_handle: tauri::AppHandle) -> Result<DiscordStatus, AppError> {
    let mut s = System::new();
    s.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());

    let discord_proc = s.processes().values().find(|p| {
        p.name()
            .to_string_lossy()
            .to_ascii_lowercase()
            .contains("discord")
    });

    let is_running = discord_proc.is_some();
    let active_pid = discord_proc.map(|p| p.pid().as_u32());

    // Elaborate installation path detection
    let installation_path =
        discord_proc.and_then(|p| p.exe().map(|path| path.to_string_lossy().to_string()));

    let mut detected_port = None;
    let rpc_available = (6463..=6472).any(|port| {
        let addr = format!("127.0.0.1:{}", port);
        if std::net::TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_millis(200))
            .is_ok()
        {
            detected_port = Some(port);
            true
        } else {
            false
        }
    });

    let browser_detected = s.processes().values().any(|p| {
        let n = p.name().to_string_lossy().to_ascii_lowercase();
        ["chrome", "firefox", "msedge", "brave"]
            .iter()
            .any(|b| n.contains(b))
    });

    // Verify system intelligence
    let _dynamic_cid = SessionAuditor::extrapolate_client_id(&app_handle);

    Ok(DiscordStatus {
        is_running,
        rpc_available,
        browser_detected,
        active_pid,
        detected_port,
        installation_path,
    })
}
