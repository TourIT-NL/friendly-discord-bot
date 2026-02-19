// src-tauri/src/auth/mod.rs

use tauri::{AppHandle, Window, Emitter, Manager};
use tokio::{sync::oneshot, io::{AsyncReadExt, AsyncWriteExt}};
use url::Url;
use std::{collections::HashMap, net::TcpListener, sync::Arc, sync::Mutex as StdMutex};
use oauth2::{
    basic::BasicClient,
    AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, TokenUrl,
    TokenResponse
};
use keyring::Entry;
use serde::{Serialize, Deserialize};
use tauri_plugin_opener::OpenerExt;
use crate::core::error::AppError;
use crate::api::rate_limiter::ApiHandle;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use futures_util::{StreamExt, SinkExt};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use tracing::{info, error, debug};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscordStatus {
    pub is_running: bool,
    pub rpc_available: bool,
    pub browser_detected: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub avatar: Option<String>,
    pub email: Option<String>,
}

const KEYRING_SERVICE: &str = "discord_privacy_util_v3"; // Version bump

#[tauri::command]
pub async fn login_with_rpc(app_handle: AppHandle, window: Window) -> Result<DiscordUser, AppError> {
    info!("Attempting Instant Link login via Discord RPC...");
    let client_id = get_discord_client_id()?;
    
    let port = (6463..=6472).find(|port| std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok());

    let port = port.ok_or_else(|| AppError {
        user_message: "Discord client not detected or RPC is disabled.".into(),
        error_code: "rpc_unavailable".into(),
        technical_details: None,
    })?;

    let url = format!("ws://127.0.0.1:{}/?v=1&client_id={}", port, client_id);
    let mut request = url.into_client_request().map_err(|e| AppError {
        user_message: "Failed to create RPC request.".into(),
        technical_details: Some(e.to_string()),
        ..Default::default()
    })?;
    request.headers_mut().insert("Origin", "https://discord.com".parse().unwrap());

    let (ws_stream, _) = connect_async(request).await?;

    let (mut write, mut read) = ws_stream.split();

    if let Some(Ok(_)) = read.next().await { debug!("RPC Handshake received"); }

    let nonce = Uuid::new_v4().to_string();
    let auth_payload = serde_json::json!({
        "cmd": "AUTHORIZE",
        "args": { "client_id": client_id, "scopes": ["identify", "guilds", "rpc"], "prompt": "none" },
        "nonce": nonce
    });
    
    write.send(Message::Text(auth_payload.to_string().into())).await?;

    while let Some(Ok(Message::Text(text))) = read.next().await {
        if let Ok(payload) = serde_json::from_str::<serde_json::Value>(&text) {
            if payload["nonce"] == nonce {
                if let Some(code) = payload["data"]["code"].as_str() {
                    let http_client = reqwest::Client::new();
                    let token_res = http_client.post("https://discord.com/api/oauth2/token")
                        .form(&[
                            ("client_id", &client_id),
                            ("client_secret", &get_discord_client_secret()?),
                            ("grant_type", &"authorization_code".to_string()),
                            ("code", &code.to_string()),
                            ("redirect_uri", &"http://127.0.0.1".to_string()),
                        ])
                        .send().await?.json::<serde_json::Value>().await?;

                    if let Some(token) = token_res["access_token"].as_str() {
                        return login_with_oauth_token(app_handle, window, token.to_string()).await;
                    }
                }
                break;
            }
        }
    }
    Err(AppError { user_message: "RPC Authorization failed. Please approve it in your Discord client.".into(), ..Default::default() })
}


#[tauri::command]
pub async fn start_qr_login_flow(app_handle: AppHandle, window: Window) -> Result<(), AppError> {
    info!("Initializing QR code login flow...");
    let url = "wss://remote-auth-gateway.discord.gg/?v=2";
    let mut request = url.into_client_request().map_err(|e| AppError {
        user_message: "Failed to create QR gateway request.".into(),
        technical_details: Some(e.to_string()),
        ..Default::default()
    })?;
    request.headers_mut().insert("Origin", "https://discord.com".parse().unwrap());
    request.headers_mut().insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".parse().unwrap());

    let (ws_stream, _) = connect_async(request).await?;

    let (_write, mut read) = ws_stream.split();

    tauri::async_runtime::spawn(async move {
        while let Some(Ok(Message::Text(text))) = read.next().await {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&text) {
                match p["op"].as_str() {
                    Some("hello") => debug!("QR Handshake OK"),
                    Some("fingerprint") => {
                        if let Some(fp) = p["fingerprint"].as_str() {
                            let _ = window.emit("qr_code_ready", format!("https://discord.com/ra/{}", fp));
                        }
                    },
                    Some("pending_remote_init") => { let _ = window.emit("qr_scanned", ()); },
                    Some("finish") => {
                        if let Some(token) = p["token"].as_str() {
                            let _ = login_with_user_token(app_handle.clone(), window.clone(), token.to_string()).await;
                        }
                        break;
                    },
                    Some("cancel") => { let _ = window.emit("qr_cancelled", ()); break; },
                    _ => {}
                }
            }
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn check_discord_status() -> Result<DiscordStatus, AppError> {
    let mut s = System::new();
    s.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    let is_running = s.processes().values().any(|p| p.name().to_string_lossy().to_ascii_lowercase().contains("discord") && !p.name().to_string_lossy().to_ascii_lowercase().contains("helper"));
    let browser_detected = s.processes().values().any(|p| ["chrome", "firefox", "msedge", "brave"].iter().any(|b| p.name().to_string_lossy().to_ascii_lowercase().contains(b)));
    let rpc_available = (6463..=6472).any(|port| std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok());
    Ok(DiscordStatus { is_running, rpc_available, browser_detected })
}

#[tauri::command]
pub async fn login_with_user_token(app_handle: AppHandle, window: Window, token: String) -> Result<DiscordUser, AppError> {
    let user_profile = validate_token(&app_handle, &token, false).await?;
    let entry = Entry::new(KEYRING_SERVICE, "discord_user")?;
    entry.set_password(&format!("TOKEN={}\nTYPE=user", token))?;
    let _ = window.emit("auth_success", user_profile.clone());
    Ok(user_profile)
}

async fn login_with_oauth_token(app_handle: AppHandle, window: Window, token: String) -> Result<DiscordUser, AppError> {
    let user_profile = validate_token(&app_handle, &token, true).await?;
    let entry = Entry::new(KEYRING_SERVICE, "discord_user")?;
    entry.set_password(&format!("TOKEN={}\nTYPE=oauth", token))?;
    let _ = window.emit("auth_success", user_profile.clone());
    Ok(user_profile)
}

async fn validate_token(app_handle: &AppHandle, token: &str, is_bearer: bool) -> Result<DiscordUser, AppError> {
    let api_handle = app_handle.state::<ApiHandle>();
    let response = api_handle.send_request(reqwest::Method::GET, "https://discord.com/api/users/@me", None, token, is_bearer).await?;
    if !response.status().is_success() {
        return Err(AppError { user_message: "Token validation failed.".into(), ..Default::default() });
    }
    Ok(response.json().await?)
}

fn get_discord_client_id() -> Result<String, AppError> {
    if let Ok(id) = std::env::var("DISCORD_CLIENT_ID") { return Ok(id); }
    Entry::new(KEYRING_SERVICE, "client_id")?.get_password().map_err(|_| AppError { user_message: "Client ID not configured.".into(), error_code: "credentials_missing".into(), ..Default::default() })
}

fn get_discord_client_secret() -> Result<String, AppError> {
    if let Ok(secret) = std::env::var("DISCORD_CLIENT_SECRET") { return Ok(secret); }
    Entry::new(KEYRING_SERVICE, "client_secret")?.get_password().map_err(|_| AppError { user_message: "Client Secret not configured.".into(), error_code: "credentials_missing".into(), ..Default::default() })
}

#[tauri::command]
pub async fn save_discord_credentials(client_id: String, client_secret: String) -> Result<(), AppError> {
    Entry::new(KEYRING_SERVICE, "client_id")?.set_password(&client_id)?;
    Entry::new(KEYRING_SERVICE, "client_secret")?.set_password(&client_secret)?;
    Ok(())
}

#[tauri::command]
pub async fn start_oauth_flow(app_handle: AppHandle, window: Window) -> Result<DiscordUser, AppError> {
    let client = BasicClient::new(ClientId::new(get_discord_client_id()?), Some(ClientSecret::new(get_discord_client_secret()?)), AuthUrl::new("https://discord.com/oauth2/authorize".to_string()).unwrap(), Some(TokenUrl::new("https://discord.com/api/oauth2/token".to_string()).unwrap()));
    let (pkce_ch, pkce_ver) = PkceCodeChallenge::new_random_sha256();
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    let redirect_url = RedirectUrl::new(format!("http://127.0.0.1:{}", port)).unwrap();
    let client = client.set_redirect_uri(redirect_url);
    let (auth_url, csrf_state) = client.authorize_url(CsrfToken::new_random).add_scope(oauth2::Scope::new("identify".into())).add_scope(oauth2::Scope::new("guilds".into())).set_pkce_challenge(pkce_ch).url();

    let (tx, rx) = oneshot::channel();
    let csrf_str = csrf_state.secret().to_string();
    tauri::async_runtime::spawn(async move {
        let listener = tokio::net::TcpListener::from_std(listener)?;
        let (mut stream, _) = listener.accept().await?;
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).await?;
        let req = String::from_utf8_lossy(&buffer[..n]);
        if let Some(code) = req.split_whitespace().nth(1).and_then(|path| Url::parse(&format!("http://localhost{}", path)).ok()).and_then(|url| url.query_pairs().find_map(|(k, v)| if k == "code" { Some(v.into_owned()) } else { None })) {
            let _ = tx.send(code);
        }
        stream.write_all(b"HTTP/1.1 200 OK\r\n\r\nAuth successful!").await?;
        Ok::<_, AppError>(())
    });

    app_handle.opener().open_url(auth_url.to_string(), None::<&str>)?;
    let code = rx.await?;
    let token_res = client.exchange_code(oauth2::AuthorizationCode::new(code)).set_pkce_verifier(PkceCodeVerifier::new(pkce_ver.secret().to_string())).request_async(oauth2::reqwest::async_http_client).await?;
    let token = token_res.access_token().secret().to_string();
    login_with_oauth_token(app_handle, window, token).await
}
