// src-tauri/src/auth/status.rs

use super::types::DiscordStatus;
use crate::core::error::AppError;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

#[tauri::command]
pub async fn check_discord_status() -> Result<DiscordStatus, AppError> {
    let mut s = System::new();
    s.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    let is_running = s.processes().values().any(|p| {
        p.name()
            .to_string_lossy()
            .to_ascii_lowercase()
            .contains("discord")
    });
    let rpc_available = (6463..=6472)
        .any(|port| std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok());
    let browser_detected = s.processes().values().any(|p| {
        let n = p.name().to_string_lossy().to_ascii_lowercase();
        ["chrome", "firefox", "msedge", "brave"]
            .iter()
            .any(|b| n.contains(b))
    });
    Ok(DiscordStatus {
        is_running,
        rpc_available,
        browser_detected,
    })
}
