// src-tauri/src/auth/rpc.rs

use crate::core::error::AppError;
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
use super::types::{DiscordUser, MASTER_CLIENT_ID, MASTER_CLIENT_SECRET};

#[tauri::command]
pub async fn login_with_rpc(
    app_handle: AppHandle,
    window: Window,
) -> Result<DiscordUser, AppError> {
    Logger::info(
        &app_handle,
        "[RPC] Initiating handshake with local Discord client.",
        None,
    );
    let client_id = Vault::get_credential(&app_handle, "client_id")
        .unwrap_or_else(|_| MASTER_CLIENT_ID.to_string());

    Logger::debug(
        &app_handle,
        &format!("[RPC] Using Client ID: {}", client_id),
        None,
    );

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
        user_message: "Discord desktop client not detected or RPC is disabled.".into(),
        ..Default::default()
    })?;

    let url = format!("ws://127.0.0.1:{}/?v=1&client_id={}", port, client_id);
    let mut request = url.into_client_request().unwrap();

    // Discord RPC sometimes requires specific Origins depending on the environment
    request
        .headers_mut()
        .insert("Origin", "https://discord.com".parse().unwrap());
    request
        .headers_mut()
        .insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".parse().unwrap());

    Logger::debug(
        &app_handle,
        "[RPC] Attempting WebSocket connection...",
        None,
    );
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
            return Err(AppError::new("Connection timed out", "rpc_timeout"));
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
            Logger::trace(
                &app_handle,
                &format!("[RPC] Raw Handshake Data: {}", text),
                None,
            );
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&text)
                && p["evt"].as_str() == Some("READY")
            {
                Logger::info(
                    &app_handle,
                    "[RPC] Link established and READY received.",
                    None,
                );
            } else {
                Logger::warn(
                    &app_handle,
                    &format!("[RPC] Unexpected initial message: {}", text),
                    None,
                );
            }
        }
        Ok(Some(Ok(m))) => Logger::warn(
            &app_handle,
            &format!("[RPC] Received non-text message during handshake: {:?}", m),
            None,
        ),
        Ok(Some(Err(e))) => Logger::error(
            &app_handle,
            "[RPC] Error reading handshake message",
            Some(serde_json::json!({"error": e.to_string()})),
        ),
        Ok(None) => Logger::warn(
            &app_handle,
            "[RPC] WebSocket closed immediately after handshake",
            None,
        ),
        Err(_) => Logger::warn(
            &app_handle,
            "[RPC] Handshake READY message timed out after 7s",
            None,
        ),
    }

    let nonce = Uuid::new_v4().to_string();
    let auth_payload = serde_json::json!({
        "cmd": "AUTHORIZE",
        "args": { "client_id": client_id, "scopes": ["identify", "guilds"], "prompt": "consent" },
        "nonce": nonce
    });

    let _ = write
        .send(Message::Text(auth_payload.to_string().into()))
        .await;

    Logger::info(
        &app_handle,
        "[RPC] Authorization request dispatched. Awaiting user consent in Discord...",
        None,
    );

    let code_res = match timeout(Duration::from_secs(60), async {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    Logger::trace(
                        &app_handle,
                        &format!("[RPC] Received Frame: {}", text),
                        None,
                    );
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
                Ok(Message::Close(f)) => {
                    Logger::warn(
                        &app_handle,
                        &format!("[RPC] Socket closed during auth: {:?}", f),
                        None,
                    );
                    return None;
                }
                Err(e) => {
                    Logger::error(
                        &app_handle,
                        "[RPC] Error in auth loop",
                        Some(serde_json::json!({"error": e.to_string()})),
                    );
                    return None;
                }
                _ => {}
            }
        }
        None
    })
    .await
    {
        Ok(Some(res)) => res,
        Ok(None) => {
            return Err(AppError::new(
                "Socket closed prematurely",
                "rpc_socket_closed",
            ));
        }
        Err(_) => {
            Logger::error(&app_handle, "[RPC] Authorization timed out after 60s", None);
            return Err(AppError {
                user_message:
                    "RPC authorization timed out. Please check Discord for the consent prompt."
                        .into(),
                ..Default::default()
            });
        }
    };

    if let Ok(mut ws_stream) = write.reunite(read) {
        let _ = timeout(Duration::from_secs(1), ws_stream.close(None)).await;
    }

    let code = code_res.map_err(|e| AppError {
        user_message: format!("RPC denied: {}", e),
        ..Default::default()
    })?;

    let client_secret = Vault::get_credential(&app_handle, "client_secret")
        .unwrap_or_else(|_| MASTER_CLIENT_SECRET.to_string());

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

    let status = response.status();
    let res_json: serde_json::Value = response.json().await?;

    if !status.is_success() {
        return Err(AppError {
            user_message: "Token exchange failed.".into(),
            technical_details: Some(res_json.to_string()),
            ..Default::default()
        });
    }

    let token = res_json["access_token"].as_str().ok_or_else(|| AppError {
        user_message: "Access token missing in response.".into(),
        ..Default::default()
    })?;
    login_with_token_internal(app_handle, window, token.to_string(), true).await
}
