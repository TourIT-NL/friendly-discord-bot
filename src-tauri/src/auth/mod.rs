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
use serde::{Serialize, Deserialize};
use tauri_plugin_opener::OpenerExt;
use crate::core::error::AppError;
use crate::core::vault::{Vault, DiscordIdentity};
use crate::api::rate_limiter::ApiHandle;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use futures_util::{StreamExt, SinkExt};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use tracing::{info, error, warn};
use uuid::Uuid;
use socket2::{Socket, Domain, Type, Protocol};
use std::net::SocketAddr;
use tokio_util::sync::CancellationToken;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct AuthState {
    qr_cancel_token: Mutex<Option<CancellationToken>>,
}

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

#[tauri::command]
pub async fn login_with_rpc(app_handle: AppHandle, window: Window) -> Result<DiscordUser, AppError> {
    info!("[RPC] Handshake sequence started.");
    let client_id = Vault::get_credential(&app_handle, "client_id")?;
    
    let port = (6463..=6472).find(|p| std::net::TcpStream::connect(format!("127.0.0.1:{}", p)).is_ok());
    let port = port.ok_or_else(|| AppError { user_message: "Discord desktop client not detected.".into(), ..Default::default() })?;

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
    
    let _ = write.send(Message::Text(auth_payload.to_string().into())).await;
    let code = match timeout(Duration::from_secs(30), async {
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                if let Ok(p) = serde_json::from_str::<serde_json::Value>(&text) {
                    if p["nonce"].as_str() == Some(&nonce) {
                        return p["data"]["code"].as_str().map(|s| s.to_string());
                    }
                }
            }
        }
        None
    }).await {
        Ok(res) => res,
        _ => return Err(AppError { user_message: "RPC authorization timed out.".into(), ..Default::default() }),
    };

    // Explicitly join and close to avoid "sending after closing" errors on drop
    let mut ws_stream = write.reunite(read).map_err(|_| AppError { user_message: "Stream internal error.".into(), ..Default::default() })?;
    let _ = timeout(Duration::from_secs(1), ws_stream.close(None)).await;

    let code = code.ok_or_else(|| AppError { user_message: "RPC authorization was denied.".into(), ..Default::default() })?;
    let client_secret = Vault::get_credential(&app_handle, "client_secret")?;
    
    let http_client = reqwest::Client::new();
    let res = http_client.post("https://discord.com/api/oauth2/token")
        .form(&[
            ("client_id", &client_id),
            ("client_secret", &client_secret),
            ("grant_type", &"authorization_code".to_string()),
            ("code", &code),
            ("redirect_uri", &"http://127.0.0.1".to_string()),
        ])
        .send().await?.json::<serde_json::Value>().await?;

    let token = res["access_token"].as_str().ok_or_else(|| AppError { user_message: "Token exchange failed.".into(), ..Default::default() })?;
    login_with_token_internal(app_handle, window, token.to_string(), true).await
}

#[tauri::command]
pub async fn start_qr_login_flow(app_handle: AppHandle, window: Window, state: tauri::State<'_, AuthState>) -> Result<(), AppError> {
    let cancel_token = CancellationToken::new();
    {
        let mut token_guard = state.qr_cancel_token.lock().await;
        if let Some(old_token) = token_guard.take() {
            old_token.cancel();
        }
        *token_guard = Some(cancel_token.clone());
    }

    let url = "wss://remote-auth-gateway.discord.gg/?v=2";
    let mut request = url.into_client_request().unwrap();
    request.headers_mut().insert("Origin", "https://discord.com".parse().unwrap());

    let (ws_stream, _) = timeout(Duration::from_secs(10), connect_async(request)).await??;
    let (_write, mut read) = ws_stream.split();
    let window_clone = window.clone();
    let app_handle_clone = app_handle.clone();

    tauri::async_runtime::spawn(async move {
        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    info!("[QR] Cancellation signal received.");
                    break;
                }
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&text) {
                                match p["op"].as_str() {
                                    Some("fingerprint") => {
                                        if let Some(fp) = p["fingerprint"].as_str() {
                                            let _ = window_clone.emit("qr_code_ready", format!("https://discord.com/ra/{}", fp));
                                        }
                                    },
                                    Some("finish") => {
                                        if let Some(token) = p["token"].as_str() {
                                            let _ = login_with_token_internal(app_handle_clone.clone(), window_clone.clone(), token.to_string(), false).await;
                                        }
                                        break;
                                    },
                                    _ => {}
                                }
                            }
                        }
                        None => break, // Connection closed
                        _ => {}
                    }
                }
            }
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn cancel_qr_login(state: tauri::State<'_, AuthState>) -> Result<(), AppError> {
    let mut token_guard = state.qr_cancel_token.lock().await;
    if let Some(token) = token_guard.take() {
        token.cancel();
    }
    Ok(())
}

#[tauri::command]
pub async fn check_discord_status() -> Result<DiscordStatus, AppError> {
    let mut s = System::new();
    s.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    let is_running = s.processes().values().any(|p| p.name().to_string_lossy().to_ascii_lowercase().contains("discord"));
    let rpc_available = (6463..=6472).any(|port| std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok());
    let browser_detected = s.processes().values().any(|p| {
        let n = p.name().to_string_lossy().to_ascii_lowercase();
        ["chrome", "firefox", "msedge", "brave"].iter().any(|b| n.contains(b))
    });
    Ok(DiscordStatus { is_running, rpc_available, browser_detected })
}

#[tauri::command]
pub async fn login_with_user_token(app_handle: AppHandle, window: Window, token: String) -> Result<DiscordUser, AppError> {
    login_with_token_internal(app_handle, window, token, false).await
}

async fn login_with_token_internal(app_handle: AppHandle, window: Window, token: String, is_oauth: bool) -> Result<DiscordUser, AppError> {
    let token = token.trim().trim_start_matches("Bearer ").trim_matches('"').to_string();
    let user_profile = validate_token(&app_handle, &token, is_oauth).await?;
    
    Vault::save_identity(&app_handle, DiscordIdentity {
        id: user_profile.id.clone(),
        username: user_profile.username.clone(),
        token: token.clone(),
        is_oauth,
    })?;
    
    let _ = window.emit("auth_success", user_profile.clone());
    Ok(user_profile)
}

async fn validate_token(app_handle: &AppHandle, token: &str, is_bearer: bool) -> Result<DiscordUser, AppError> {
    let api_handle = app_handle.state::<ApiHandle>();
    let response = api_handle.send_request(reqwest::Method::GET, "https://discord.com/api/v9/users/@me", None, token, is_bearer).await?;
    if !response.status().is_success() {
        return Err(AppError { user_message: "Token invalid or expired.".into(), ..Default::default() });
    }
    Ok(response.json().await?)
}

#[tauri::command]
pub async fn save_discord_credentials(app_handle: AppHandle, client_id: String, client_secret: String) -> Result<(), AppError> {
    Vault::set_credential(&app_handle, "client_id", client_id.trim())?;
    Vault::set_credential(&app_handle, "client_secret", client_secret.trim())?;
    Ok(())
}

#[tauri::command]
pub async fn list_identities(app_handle: AppHandle) -> Result<Vec<DiscordIdentity>, AppError> {
    Ok(Vault::list_identities(&app_handle))
}

#[tauri::command]
pub async fn switch_identity(app_handle: AppHandle, window: Window, id: String) -> Result<DiscordUser, AppError> {
    let identities = Vault::list_identities(&app_handle);
    let identity = identities.iter().find(|i| i.id == id).ok_or_else(|| AppError { user_message: "Identity not found.".into(), ..Default::default() })?;
    login_with_token_internal(app_handle, window, identity.token.clone(), identity.is_oauth).await
}

#[tauri::command]
pub async fn remove_identity(app_handle: AppHandle, id: String) -> Result<(), AppError> {
    Vault::remove_identity(&app_handle, &id)
}

#[tauri::command]
pub async fn start_oauth_flow(app_handle: AppHandle, window: Window) -> Result<DiscordUser, AppError> {
    let client_id = Vault::get_credential(&app_handle, "client_id")?;
    let client_secret = Vault::get_credential(&app_handle, "client_secret")?;
    
    let client = BasicClient::new(
        ClientId::new(client_id), 
        Some(ClientSecret::new(client_secret)), 
        AuthUrl::new("https://discord.com/oauth2/authorize".to_string()).unwrap(), 
        Some(TokenUrl::new("https://discord.com/api/v9/oauth2/token".to_string()).unwrap())
    );
    
    let (pkce_ch, pkce_ver) = PkceCodeChallenge::new_random_sha256();
    let port = 58123;
    let client = client.set_redirect_uri(RedirectUrl::new(format!("http://127.0.0.1:{}", port)).unwrap());
    let (auth_url, csrf) = client.authorize_url(CsrfToken::new_random)
        .add_scope(oauth2::Scope::new("identify".into()))
        .add_scope(oauth2::Scope::new("guilds".into()))
        .set_pkce_challenge(pkce_ch)
        .url();

    let (tx, rx) = oneshot::channel::<String>();
    let csrf_secret = csrf.secret().clone();
    
    let addr = format!("127.0.0.1:{}", port).parse::<SocketAddr>().map_err(|e| AppError { user_message: "Invalid bind address.".into(), technical_details: Some(e.to_string()), ..Default::default() })?;
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
    socket.set_reuse_address(true)?;
    #[cfg(not(windows))]
    socket.set_reuse_port(true)?;
    socket.bind(&addr.into())?;
    socket.listen(128)?;
    let listener: std::net::TcpListener = socket.into();

    tauri::async_runtime::spawn(async move {
        let listener = tokio::net::TcpListener::from_std(listener)?;
        if let Ok(Ok((mut stream, _))) = timeout(Duration::from_secs(120), listener.accept()).await {
            let mut buffer = [0; 2048];
            let n = stream.read(&mut buffer).await?;
            let req = String::from_utf8_lossy(&buffer[..n]);
            if let Some(url) = req.split_whitespace().nth(1).and_then(|p| Url::parse(&format!("http://localhost{}", p)).ok()) {
                let query: HashMap<String, String> = url.query_pairs().into_owned().collect();
                if query.get("state").map(|s| s == &csrf_secret).unwrap_or(false) {
                    if let Some(code) = query.get("code") { let _ = tx.send(code.clone()); }
                }
            }
            let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body style='font-family:sans-serif; text-align:center; padding-top:50px;'><h1>Handshake Successful</h1><p>You can close this tab and return to the app.</p></body></html>";
            let _ = stream.write_all(response.as_bytes()).await;
        }
        Ok::<_, AppError>(())
    });

    app_handle.opener().open_url(auth_url.to_string(), None::<&str>)?;
    let code = match timeout(Duration::from_secs(120), rx).await {
        Ok(Ok(c)) => c,
        _ => return Err(AppError { user_message: "Authorization timed out.".into(), ..Default::default() }),
    };
    
    let token_res = client.exchange_code(oauth2::AuthorizationCode::new(code))
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_ver.secret().to_string()))
        .request_async(oauth2::reqwest::async_http_client).await?;
        
    login_with_token_internal(app_handle, window, token_res.access_token().secret().to_string(), true).await
}

#[tauri::command]
pub async fn get_current_user(app_handle: AppHandle, window: Window) -> Result<DiscordUser, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let user_profile = validate_token(&app_handle, &token, is_bearer).await?;
    let _ = window.emit("auth_success", user_profile.clone());
    Ok(user_profile)
}
