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
use super::types::DiscordUser;

#[tauri::command]
pub async fn login_with_rpc(
    app_handle: AppHandle,
    window: Window,
) -> Result<DiscordUser, AppError> {
    Logger::info(&app_handle, "[RPC] Handshake sequence started.", None);
    let client_id = match Vault::get_credential(&app_handle, "client_id") {
        Ok(id) => id,
        Err(e) => return Err(e),
    };

    let port =
        (6463..=6472).find(|p| std::net::TcpStream::connect(format!("127.0.0.1:{}", p)).is_ok());
    let port = port.ok_or_else(|| AppError {
        user_message: "Discord desktop client not detected.".into(),
        ..Default::default()
    })?;

    let url = format!("ws://127.0.0.1:{}/?v=1&client_id={}", port, client_id);
    let mut request = url.into_client_request().unwrap();
    request
        .headers_mut()
        .insert("Origin", "https://discord.com".parse().unwrap());

    let (ws_stream, _) = timeout(Duration::from_secs(5), connect_async(request)).await??;
    let (mut write, mut read) = ws_stream.split();

    // Wait for DISPATCH READY
    if let Ok(Some(Ok(Message::Text(text)))) = timeout(Duration::from_secs(2), read.next()).await {
        if let Ok(p) = serde_json::from_str::<serde_json::Value>(&text) {
            if p["evt"].as_str() == Some("READY") {
                Logger::debug(
                    &app_handle,
                    "[RPC] Link established with desktop client",
                    None,
                );
            }
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

    let code_res = match timeout(Duration::from_secs(30), async {
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                if let Ok(p) = serde_json::from_str::<serde_json::Value>(&text) {
                    if p["nonce"].as_str() == Some(&nonce) {
                        if let Some(err) = p["data"]["message"].as_str() {
                            return Some(Err(err.to_string()));
                        }
                        return p["data"]["code"].as_str().map(|s| Ok(s.to_string()));
                    }
                }
            }
        }
        None
    })
    .await
    {
        Ok(Some(res)) => res,
        _ => {
            return Err(AppError {
                user_message: "RPC authorization timed out.".into(),
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

    let client_secret = Vault::get_credential(&app_handle, "client_secret")?;

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
