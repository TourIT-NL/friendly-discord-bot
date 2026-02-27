// src-tauri/src/auth/rpc.rs

use crate::core::error::AppError;
use crate::core::forensics::auditor::SessionAuditor;
use crate::core::logger::Logger;
use crate::core::vault::Vault;
use futures_util::{SinkExt, StreamExt};
use serde_json;
use tauri::{AppHandle, Window};
use tokio::time::{Duration, timeout};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::Message;
use uuid::Uuid;

use super::identity::login_with_token_internal;
use super::types::DiscordUser;

#[tauri::command]
pub async fn login_with_rpc(
    app_handle: AppHandle,
    window: Window,
) -> Result<DiscordUser, AppError> {
    Logger::info(
        &app_handle,
        "[Auth] Initiating native client extrapolation...",
        None,
    );

    // 1. Primary Strategy: Direct Token Extrapolation (Zero-Network, No Secrets)
    if let Ok(token) = SessionAuditor::extrapolate_token(&app_handle) {
        Logger::info(
            &app_handle,
            "[Auth] Extrapolated active session via forensics.",
            None,
        );
        return login_with_token_internal(app_handle, window, token, false).await;
    }

    // 2. Fallback Strategy: RPC WebSocket Handshake
    Logger::info(
        &app_handle,
        "[RPC] Falling back to WebSocket handshake.",
        None,
    );
    let client_id = Vault::get_credential(&app_handle, "client_id")
        .unwrap_or_else(|_| SessionAuditor::extrapolate_client_id(&app_handle));

    let port = (6463..=6472).find(|p| {
        let addr = format!("127.0.0.1:{}", p);
        Logger::trace(&app_handle, &format!("[RPC] Probing port {}...", p), None);
        match std::net::TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            Duration::from_millis(1000),
        ) {
            Ok(_) => {
                Logger::info(
                    &app_handle,
                    &format!("[RPC] Active listener detected on port {}", p),
                    None,
                );
                true
            }
            Err(e) => {
                Logger::trace(
                    &app_handle,
                    &format!("[RPC] Port {} inactive: {}", p, e),
                    None,
                );
                false
            }
        }
    });

    let port = port.ok_or_else(|| AppError {
        user_message: "Discord desktop client not detected. Please ensure Discord is running and you are logged in.".into(),
        ..Default::default()
    })?;

    let url = format!("ws://127.0.0.1:{}/?v=1&client_id={}", port, client_id);
    let mut request = url.into_client_request().unwrap();

    request
        .headers_mut()
        .insert("Origin", "https://discord.com".parse().unwrap());
    request
        .headers_mut()
        .insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".parse().unwrap());

    let ws_connect_result = timeout(Duration::from_secs(10), connect_async(request)).await;

    let (ws_stream, _) = match ws_connect_result {
        Ok(Ok(stream)) => stream,
        Ok(Err(e)) => {
            Logger::error(
                &app_handle,
                "[RPC] WebSocket connection failed",
                Some(serde_json::json!({"error": e.to_string()})),
            );
            return Err(AppError::from(e));
        }
        Err(_) => {
            Logger::error(&app_handle, "[RPC] Connection timed out after 10s", None);
            return Err(AppError::new(
                "Native handshake timed out. Try the QR method.",
                "rpc_timeout",
            ));
        }
    };

    let (mut write, mut read) = ws_stream.split();

    // Wait for DISPATCH READY
    Logger::debug(
        &app_handle,
        "[RPC] Awaiting READY event from client...",
        None,
    );
    let ready_res = timeout(Duration::from_secs(7), read.next()).await;
    match ready_res {
        Ok(Some(Ok(Message::Text(text)))) => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&text)
                && p["evt"].as_str() == Some("READY")
            {
                Logger::info(
                    &app_handle,
                    "[RPC] Link established and READY received.",
                    None,
                );
            }
        }
        _ => {
            Logger::warn(
                &app_handle,
                "[RPC] READY event not received or malformed.",
                None,
            );
        }
    }

    let nonce = Uuid::new_v4().to_string();
    let auth_payload = serde_json::json!({
        "cmd": "AUTHORIZE",
        "args": { "client_id": client_id, "scopes": ["identify", "guilds"], "prompt": "none" },
        "nonce": nonce
    });

    let _ = write
        .send(Message::Text(auth_payload.to_string().into()))
        .await;

    Logger::info(
        &app_handle,
        "[RPC] Authorization request dispatched. Please approve in Discord if prompted.",
        None,
    );

    let code_res = match timeout(Duration::from_secs(60), async {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(p) = serde_json::from_str::<serde_json::Value>(&text)
                        && p["nonce"].as_str() == Some(&nonce)
                    {
                        if let Some(err) = p["data"]["message"].as_str() {
                            return Some(Err(err.to_string()));
                        }
                        if let Some(code) = p["data"]["code"].as_str() {
                            return Some(Ok(code.to_string()));
                        }
                    }
                }
                _ => {}
            }
        }
        None
    })
    .await
    {
        Ok(Some(res)) => res,
        _ => {
            return Err(AppError {
                user_message: "RPC authorization timed out. Direct extrapolation failed and RPC consent was not received.".into(),
                ..Default::default()
            });
        }
    };

    if let Ok(mut ws_stream) = write.reunite(read) {
        let _ = timeout(Duration::from_secs(1), ws_stream.close(None)).await;
    }

    let code = code_res.map_err(|e| AppError {
        user_message: format!("RPC access denied: {}", e),
        ..Default::default()
    })?;

    // Note: If code exchange requires a secret and it's missing, we inform the user.
    let client_secret = match Vault::get_credential(&app_handle, "client_secret") {
        Ok(s) => s,
        Err(_) => {
            return Err(AppError {
                user_message: "Authorization code received, but a Client Secret is required for final exchange. Please complete Setup or use the QR method.".into(),
                error_code: "vault_credentials_missing".into(),
                ..Default::default()
            });
        }
    };

    let http_client = reqwest::Client::new();
    let response = http_client
        .post("https://discord.com/api/oauth2/token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("grant_type", "authorization_code"),
            ("code", code.as_str()),
            ("redirect_uri", "http://127.0.0.1"),
        ])
        .send()
        .await?;

    let res_json: serde_json::Value = response.json().await?;
    let token = res_json["access_token"].as_str().ok_or_else(|| AppError {
        user_message: "Access token missing in response.".into(),
        ..Default::default()
    })?;

    login_with_token_internal(app_handle, window, token.to_string(), true).await
}
