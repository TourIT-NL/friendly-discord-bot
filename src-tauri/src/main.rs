// src-tauri/src/main.rs

mod auth;
mod core;
mod api;

use tauri::Manager;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::sync::mpsc;
use crate::api::rate_limiter::{RateLimiterActor, ApiHandle};
use crate::core::op_manager::OperationManager;

fn main() {
    if let Err(e) = rustls::crypto::ring::default_provider().install_default() {
        eprintln!("Failed to install rustls default provider: {:?}", e);
        // Do not exit here, might not be critical for all features
    }

    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_local_data_dir().expect("failed to get app dir");
            std::fs::create_dir_all(&app_data_dir).expect("failed to create app dir");

            let file_appender = tracing_appender::rolling::daily(&app_data_dir, "app.log");
            let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

            app.manage(_guard);

            // Logging to both stdout and file. Remove one of the following `.with` lines if you want to avoid duplicate log entries.
            tracing_subscriber::registry()
                .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
                .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
                .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
                .init();

            info!("Application starting up...");

            let (tx, rx) = mpsc::channel(100);
            let mut rate_limiter = RateLimiterActor::new(rx); // No need for 'mut' since it's moved into spawn
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
            api::discord::get_operation_status
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| panic!("error while running tauri application: {:?}", e));
}
