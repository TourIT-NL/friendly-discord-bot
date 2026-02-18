mod auth;
mod core;
mod api;

use tauri::Manager;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::sync::mpsc;
use crate::api::rate_limiter::{RateLimiterActor, ApiHandle};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Initialize logging
            let app_data_dir = app.path().app_local_data_dir()
                .expect("failed to get app local data directory");
            
            // Ensure the directory exists
            std::fs::create_dir_all(&app_data_dir)
                .expect("failed to create app local data directory");

            let file_appender = tracing_appender::rolling::daily(&app_data_dir, "app.log");
            let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
            
            app.manage(_guard);

            tracing_subscriber::registry()
                .with(tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info".into()))
                .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
                .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
                .init();

            info!("Application starting up...");

            // Initialize the RateLimiterActor
            let (tx, rx) = mpsc::channel(100);
            let mut rate_limiter = RateLimiterActor::new(rx);
            let api_handle = ApiHandle::new(tx);
            
            // Spawn the actor
            tokio::spawn(async move {
                rate_limiter.run().await;
            });

            // Store the handle in app state
            app.manage(api_handle);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth::start_oauth_flow,
            api::discord::fetch_guilds,
            api::discord::fetch_channels,
            api::discord::bulk_delete_messages
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}