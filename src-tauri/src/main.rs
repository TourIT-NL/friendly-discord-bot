// src-tauri/src/main.rs

mod api;
mod auth;
mod core;

use crate::api::rate_limiter::{ApiHandle, RateLimiterActor};
use crate::core::op_manager::OperationManager;
use tauri::Manager;
use tokio::sync::mpsc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(target_os = "windows")]
#[allow(dead_code)] // Added to suppress unused function warning in debug builds
fn ensure_elevation() {
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOW;

    if !is_elevated::is_elevated() {
        let exe_path = std::env::current_exe().expect("Failed to get current exe path");
        let exe_path_wide: Vec<u16> = exe_path.as_os_str().encode_wide().chain(Some(0)).collect();
        let verb_wide: Vec<u16> = "runas".encode_utf16().chain(Some(0)).collect();

        unsafe {
            ShellExecuteW(
                0 as HWND,
                verb_wide.as_ptr(),
                exe_path_wide.as_ptr(),
                ptr::null(),
                ptr::null(),
                SW_SHOW,
            );
        }
        std::process::exit(0);
    }
}

fn main() {
    #[cfg(all(target_os = "windows", not(debug_assertions)))]
    ensure_elevation();

    if let Err(e) = rustls::crypto::ring::default_provider().install_default() {
        eprintln!(
            "WARN: Failed to install rustls default provider: {:?}. Secure communication might be impacted for some features.",
            e
        );
        // If rustls is critical, consider exiting or providing a strong user warning.
        // For now, adhere to previous decision not to exit.
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_local_data_dir()
                .expect("failed to get app dir");
            std::fs::create_dir_all(&app_data_dir).expect("failed to create app dir");

            let file_appender = tracing_appender::rolling::daily(&app_data_dir, "app.log");
            let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

            app.manage(_guard);

            // Logging to both stdout and file.
            let env_filter =
                tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                    #[cfg(debug_assertions)]
                    {
                        "src_tauri=debug,info".into()
                    }
                    #[cfg(not(debug_assertions))]
                    {
                        "src_tauri=info".into()
                    }
                });

            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
                .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
                .init();

            info!("Application starting up...");

            let (tx, rx) = mpsc::channel(100);
            let mut rate_limiter = RateLimiterActor::new(rx, app.handle().clone());
            let api_handle = ApiHandle::new(tx);

            tauri::async_runtime::spawn(async move {
                rate_limiter.run().await;
            });

            app.manage(api_handle);

            let op_manager = OperationManager::new();
            app.manage(op_manager);

            let auth_state = auth::AuthState::default();
            app.manage(auth_state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth::start_oauth_flow,
            auth::save_discord_credentials,
            auth::check_discord_status,
            auth::login_with_user_token,
            auth::start_qr_login_flow,
            auth::cancel_qr_login,
            auth::login_with_rpc,
            auth::get_current_user,
            auth::list_identities,
            auth::switch_identity,
            auth::remove_identity,
            api::discord::fetch_guilds,
            api::discord::fetch_channels,
            api::discord::fetch_relationships,
            api::discord::fetch_preview_messages,
            api::discord::bulk_delete_messages,
            api::discord::bulk_leave_guilds,
            api::discord::bulk_remove_relationships,
            api::discord::stealth_privacy_wipe,
            api::discord::bury_audit_log,
            api::discord::webhook_ghosting,
            api::discord::nitro_stealth_wipe,
            api::discord::pause_operation,
            api::discord::resume_operation,
            api::discord::abort_operation,
            api::discord::get_operation_status,
            api::discord::tools::open_external_link
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| panic!("error while running tauri application: {:?}", e));
}
