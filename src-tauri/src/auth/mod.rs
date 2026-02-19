// src-tauri/src/auth/mod.rs

use tauri::{AppHandle, Window, Emitter, Manager};
use tokio::{sync::oneshot, io::{AsyncReadExt, AsyncWriteExt}, time::{timeout, Duration}};
use url::Url;
use std::net::TcpListener;
use std::collections::HashMap;
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
use futures_util::StreamExt;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use tracing::info;
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

pub const KEYRING_SERVICE: &str = "discord_privacy_util_v_final_stable";

#[tauri::command]
pub async fn get_current_user(app_handle: AppHandle, window: Window) -> Result<DiscordUser, AppError> {
    let (token, is_bearer) = get_stored_token_internal()?;
    let user_profile = validate_token(&app_handle, &token, is_bearer).await?;
    let _ = window.emit("auth_success", user_profile.clone());
    Ok(user_profile)
}

fn get_stored_token_internal() -> Result<(String, bool), AppError> {
    let entry = Entry::new(KEYRING_SERVICE, "discord_user")?;
    let password = entry.get_password()?;
    
    let token = password.lines()
        .find(|line| line.starts_with("TOKEN="))
        .and_then(|line| line.strip_prefix("TOKEN="))
        .ok_or_else(|| AppError { user_message: "No session.".into(), ..Default::default() })?;

    let is_bearer = password.lines()
        .find(|line| line.starts_with("TYPE="))
        .map(|line| line.contains("oauth"))
        .unwrap_or(false);

    Ok((token.to_string(), is_bearer))
}

#[tauri::command]
pub async fn login_with_rpc(app_handle: AppHandle, window: Window) -> Result<DiscordUser, AppError> {
    info!("[RPC] Handshake init.");
    let client_id = get_discord_client_id()?;
    
    let port = (6463..=6472).find(|p| std::net::TcpStream::connect(format!("127.0.0.1:{}", p)).is_ok());
    let port = port.ok_or_else(|| AppError { user_message: "Discord not detected.".into(), ..Default::default() })?;

    let url = format!("ws://127.0.0.1:{}/?v=1&client_id={}", port, client_id);
    let mut request = url.into_client_request().unwrap();
    request.headers_mut().insert("Origin", "https://discord.com".parse().unwrap());

    let (ws_stream, _) = timeout(Duration::from_secs(5), connect_async(request)).await??;
    let (mut write, mut read) = ws_stream.split();

    let _ = timeout(Duration::from_secs(2), read.next()).await;

    let nonce = Uuid::new_v4().to_string();
    let auth_payload = serde_json::json!({
        "cmd": "AUTHORIZE",
        "args": { "client_id": client_id, "scopes": ["identify", "guilds"], "prompt": "none" },
        "nonce": nonce
    });
    
    use futures_util::SinkExt;
    write.send(Message::Text(auth_payload.to_string().into())).await?;

    let code = match timeout(Duration::from_secs(30), async {
        while let Some(Ok(Message::Text(text))) = read.next().await {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&text) {
                if p["nonce"].as_str() == Some(&nonce) {
                    return p["data"]["code"].as_str().map(|s| s.to_string());
                }
            }
        }
        None
    }).await {
        Ok(res) => res,
        _ => return Err(AppError { user_message: "RPC timed out.".into(), ..Default::default() }),
    };

    let code = code.ok_or_else(|| AppError { user_message: "RPC denied.".into(), ..Default::default() })?;
    
    let http_client = reqwest::Client::new();
    let res = http_client.post("https://discord.com/api/oauth2/token")
        .form(&[
            ("client_id", &client_id),
            ("client_secret", &get_discord_client_secret()?),
            ("grant_type", &"authorization_code".to_string()),
            ("code", &code),
            ("redirect_uri", &"http://127.0.0.1".to_string()),
        ])
        .send().await?.json::<serde_json::Value>().await?;

    let token = res["access_token"].as_str().ok_or_else(|| AppError { user_message: "Handshake failed.".into(), ..Default::default() })?;
    login_with_oauth_token(app_handle, window, token.to_string()).await
}

#[tauri::command]
pub async fn start_qr_login_flow(app_handle: AppHandle, window: Window) -> Result<(), AppError> {
    info!("[QR] Connecting...");
    let url = "wss://remote-auth-gateway.discord.gg/?v=2";
    let mut request = url.into_client_request().unwrap();
    request.headers_mut().insert("Origin", "https://discord.com".parse().unwrap());
    request.headers_mut().insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".parse().unwrap());

    let (ws_stream, _) = timeout(Duration::from_secs(10), connect_async(request)).await??;
    let (_write, mut read) = ws_stream.split();
    let window_clone = window.clone();
    let app_handle_clone = app_handle.clone();

    tauri::async_runtime::spawn(async move {
        while let Some(Ok(Message::Text(text))) = read.next().await {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&text) {
                match p["op"].as_str() {
                    Some("fingerprint") => {
                        if let Some(fp) = p["fingerprint"].as_str() {
                            let _ = window_clone.emit("qr_code_ready", format!("https://discord.com/ra/{}", fp));
                        }
                    },
                    Some("pending_remote_init") => { let _ = window_clone.emit("qr_scanned", ()); },
                    Some("finish") => {
                        if let Some(token) = p["token"].as_str() {
                            let _ = login_with_user_token(app_handle_clone.clone(), window_clone.clone(), token.to_string()).await;
                        }
                        break;
                    },
                    Some("cancel") => { let _ = window_clone.emit("qr_cancelled", ()); break; },
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
    let is_running = s.processes().values().any(|p| p.name().to_string_lossy().to_ascii_lowercase().contains("discord"));
    let browser_detected = s.processes().values().any(|p| {
        let n = p.name().to_string_lossy().to_ascii_lowercase();
        ["chrome", "firefox", "msedge", "brave"].iter().any(|b| n.contains(b))
    });
    let rpc_available = (6463..=6472).any(|port| std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok());
    Ok(DiscordStatus { is_running, rpc_available, browser_detected })
}

#[tauri::command]
pub async fn login_with_user_token(app_handle: AppHandle, window: Window, token: String) -> Result<DiscordUser, AppError> {
    let token = token.trim().trim_start_matches("Bearer ").trim_matches('"').to_string();
    let user_profile = validate_token(&app_handle, &token, false).await?;
    Entry::new(KEYRING_SERVICE, "discord_user")?.set_password(&format!("TOKEN={}\nTYPE=user", token))?;
    let _ = window.emit("auth_success", user_profile.clone());
    Ok(user_profile)
}

pub async fn login_with_oauth_token(app_handle: AppHandle, window: Window, token: String) -> Result<DiscordUser, AppError> {
    let user_profile = validate_token(&app_handle, &token, true).await?;
    Entry::new(KEYRING_SERVICE, "discord_user")?.set_password(&format!("TOKEN={}\nTYPE=oauth", token))?;
    let _ = window.emit("auth_success", user_profile.clone());
    Ok(user_profile)
}

async fn validate_token(app_handle: &AppHandle, token: &str, is_bearer: bool) -> Result<DiscordUser, AppError> {
    let api_handle = app_handle.state::<ApiHandle>();
    let response = api_handle.send_request(reqwest::Method::GET, "https://discord.com/api/users/@me", None, token, is_bearer).await?;
    if !response.status().is_success() {
        return Err(AppError { user_message: "Invalid token.".into(), ..Default::default() });
    }
    Ok(response.json().await?)
}

fn get_discord_client_id() -> Result<String, AppError> {
    if let Ok(id) = std::env::var("DISCORD_CLIENT_ID") { return Ok(id); }
    Entry::new(KEYRING_SERVICE, "client_id")?.get_password().map_err(|_| AppError { user_message: "Config missing.".into(), error_code: "credentials_missing".into(), ..Default::default() })
}

fn get_discord_client_secret() -> Result<String, AppError> {
    if let Ok(secret) = std::env::var("DISCORD_CLIENT_SECRET") { return Ok(secret); }
    Entry::new(KEYRING_SERVICE, "client_secret")?.get_password().map_err(|_| AppError { user_message: "Config missing.".into(), error_code: "credentials_missing".into(), ..Default::default() })
}

#[tauri::command]
pub async fn save_discord_credentials(client_id: String, client_secret: String) -> Result<(), AppError> {
    Entry::new(KEYRING_SERVICE, "client_id")?.set_password(client_id.trim())?;
    Entry::new(KEYRING_SERVICE, "client_secret")?.set_password(client_secret.trim())?;
    Ok(())
}

#[tauri::command]
pub async fn start_oauth_flow(app_handle: AppHandle, window: Window) -> Result<DiscordUser, AppError> {
    let client_id = get_discord_client_id()?;
    let client_secret = get_discord_client_secret()?;
    let client = BasicClient::new(ClientId::new(client_id), Some(ClientSecret::new(client_secret)), AuthUrl::new("https://discord.com/oauth2/authorize".to_string()).unwrap(), Some(TokenUrl::new("https://discord.com/api/oauth2/token".to_string()).unwrap()));
    let (pkce_ch, pkce_ver) = PkceCodeChallenge::new_random_sha256();
    let csrf = CsrfToken::new_random();
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    let client = client.set_redirect_uri(RedirectUrl::new(format!("http://127.0.0.1:{}", port)).unwrap());
    let (auth_url, _) = client.authorize_url(|| csrf.clone()).add_scope(oauth2::Scope::new("identify".into())).add_scope(oauth2::Scope::new("guilds".into())).set_pkce_challenge(pkce_ch).url();

    let (tx, rx) = oneshot::channel::<String>();
    let csrf_secret = csrf.secret().clone();
    tauri::async_runtime::spawn(async move {
        let listener = tokio::net::TcpListener::from_std(listener)?;
        if let Ok(Ok((mut stream, _))) = timeout(Duration::from_secs(60), listener.accept()).await {
            let mut buffer = [0; 1024];
            let n = stream.read(&mut buffer).await?;
            let req = String::from_utf8_lossy(&buffer[..n]);
            if let Some(url) = req.split_whitespace().nth(1).and_then(|p| Url::parse(&format!("http://localhost{}", p)).ok()) {
                let query: HashMap<String, String> = url.query_pairs().into_owned().collect();
                if query.get("state").map(|s| s == &csrf_secret).unwrap_or(false) {
                    if let Some(code) = query.get("code") {
                        let _ = tx.send(code.clone());
                    }
                }
            }
            let _ = stream.write_all(b"HTTP/1.1 200 OK\r\n\r\nHandshake success. Return to app.").await;
        }
        Ok::<_, AppError>(())
    });

    app_handle.opener().open_url(auth_url.to_string(), None::<&str>)?;
    let code = rx.await.map_err(|_| AppError { user_message: "Auth timed out.".into(), ..Default::default() })?;
    let token_res = client.exchange_code(oauth2::AuthorizationCode::new(code)).set_pkce_verifier(PkceCodeVerifier::new(pkce_ver.secret().to_string())).request_async(oauth2::reqwest::async_http_client).await?;
    let token = token_res.access_token().secret().to_string();
    login_with_oauth_token(app_handle, window, token).await
}
