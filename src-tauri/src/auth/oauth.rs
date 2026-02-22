// src-tauri/src/auth/oauth.rs

use oauth2::{
    AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl,
    TokenResponse, TokenUrl, basic::BasicClient,
};
use socket2::{Domain, Protocol, Socket, Type};
use std::collections::HashMap;
use std::net::SocketAddr;
use tauri::{AppHandle, Window};
use tauri_plugin_opener::OpenerExt;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::oneshot,
    time::{Duration, timeout},
};
use url::Url;

use super::identity::login_with_token_internal;
use super::types::DiscordUser;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::vault::Vault;

#[tauri::command]
pub async fn start_oauth_flow(
    app_handle: AppHandle,
    window: Window,
) -> Result<DiscordUser, AppError> {
    Logger::info(&app_handle, "[OAuth] Starting official flow...", None);
    let client_id = match Vault::get_credential(&app_handle, "client_id") {
        Ok(id) => id,
        Err(e) if e.error_code == "vault_credentials_missing" => return Err(e),
        Err(e) => return Err(e),
    };
    let client_secret = match Vault::get_credential(&app_handle, "client_secret") {
        Ok(secret) => secret,
        Err(e) if e.error_code == "vault_credentials_missing" => return Err(e),
        Err(e) => return Err(e),
    };

    let client = BasicClient::new(
        ClientId::new(client_id.clone()),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new("https://discord.com/oauth2/authorize".to_string()).unwrap(),
        Some(TokenUrl::new("https://discord.com/api/v10/oauth2/token".to_string()).unwrap()),
    );

    let (pkce_ch, pkce_ver) = PkceCodeChallenge::new_random_sha256();
    let port = 58123; // Fixed port
    let client =
        client.set_redirect_uri(RedirectUrl::new(format!("http://127.0.0.1:{}", port)).unwrap());
    let (auth_url, csrf) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(oauth2::Scope::new("identify".into()))
        .add_scope(oauth2::Scope::new("guilds".into()))
        .set_pkce_challenge(pkce_ch)
        .url();

    let (tx, rx) = oneshot::channel::<String>();
    let csrf_secret = csrf.secret().clone();

    let addr = format!("127.0.0.1:{}", port)
        .parse::<SocketAddr>()
        .map_err(|e| AppError {
            user_message: "Invalid bind address.".into(),
            technical_details: Some(e.to_string()),
            ..Default::default()
        })?;

    Logger::debug(
        &app_handle,
        &format!("[OAuth] Binding callback to {}", addr),
        None,
    );
    let mut socket = None;
    for i in 0..3 {
        let s = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
        s.set_reuse_address(true)?;
        #[cfg(not(windows))]
        s.set_reuse_port(true)?;
        if s.bind(&addr.into()).is_ok() {
            s.listen(128)?;
            socket = Some(s);
            break;
        }
        Logger::warn(
            &app_handle,
            &format!("[OAuth] Port busy, retrying... ({}/3)", i + 1),
            None,
        );
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    let socket = socket.ok_or_else(|| {
        Logger::error(&app_handle, "[OAuth] Failed to bind callback port", None);
        AppError {
            user_message: "Authorization port (58123) is already in use.".into(),
            ..Default::default()
        }
    })?;

    let listener: std::net::TcpListener = socket.into(); // Revert back to original listener variable name
    let app_clone = app_handle.clone(); // Revert app_clone position

    tauri::async_runtime::spawn(async move {
        let listener = tokio::net::TcpListener::from_std(listener)?;
        if let Ok(Ok((mut stream, _))) = timeout(Duration::from_secs(120), listener.accept()).await
        {
            let mut buffer = [0; 4096];
            let n = stream.read(&mut buffer).await?;
            let req = String::from_utf8_lossy(&buffer[..n]);

            // Robust request parsing
            let first_line = req.lines().next().unwrap_or_default();
            if let Some(path) = first_line.split_whitespace().nth(1)
                && let Ok(url) = Url::parse(&format!("http://127.0.0.1{}", path)) {
                    let query: HashMap<String, String> = url.query_pairs().into_owned().collect();
                    if query
                        .get("state")
                        .map(|s| s == &csrf_secret)
                        .unwrap_or(false)
                    {
                        if let Some(code) = query.get("code") {
                            let _ = tx.send(code.clone());
                        }
                    } else {
                        Logger::warn(&app_clone, "[OAuth] CSRF state mismatch or missing", None);
                    }
            }

            let response = "HTTP/1.1 200 OK
Content-Type: text/html

<html><body style='font-family:sans-serif; text-align:center; padding-top:100px; background:#0a0a0a; color:white;'><h1>Handshake Successful</h1><p>Auth signature verified. You can return to the application.</p></body></html>";
            if let Err(e) = stream.write_all(response.as_bytes()).await {
                Logger::error(
                    &app_clone,
                    "[OAuth] Failed to send success response",
                    Some(serde_json::json!({"error": e.to_string()})),
                );
            }
            let _ = stream.flush().await;
            let _ = stream.shutdown().await;
        }
        Ok::<_, AppError>(())
    });

    Logger::debug(&app_handle, "[OAuth] Opening browser gateway...", None);
    app_handle
        .opener()
        .open_url(auth_url.to_string(), None::<&str>)?;

    let code = match timeout(Duration::from_secs(120), rx).await {
        Ok(Ok(c)) => c,
        _ => {
            Logger::error(
                &app_handle,
                "[OAuth] Authorization timed out (User cancelled or network blocked callback)",
                None,
            );
            return Err(AppError {
                user_message: "Authorization timed out.".into(),
                ..Default::default()
            });
        }
    };

    Logger::debug(
        &app_handle,
        "[OAuth] Code received. Verifying PKCE and exchanging...",
        None,
    );
    let token_res = client
        .exchange_code(oauth2::AuthorizationCode::new(code))
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_ver.secret().to_string()))
        .request_async(oauth2::reqwest::async_http_client)
        .await;

    match token_res {
        Ok(res) => {
            Logger::info(&app_handle, "[OAuth] Token exchange successful", None);
            login_with_token_internal(
                app_handle,
                window,
                res.access_token().secret().to_string(),
                true,
            )
            .await
        }
        Err(e) => {
            Logger::error(
                &app_handle,
                "[OAuth] Token exchange failed",
                Some(serde_json::json!({ "error": format!("{:?}", e) })),
            );
            Err(AppError {
                user_message: "Exchange protocol failure.".into(),
                ..Default::default()
            })
        }
    }
}
