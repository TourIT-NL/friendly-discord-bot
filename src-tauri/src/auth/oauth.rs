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
    let client_id = Vault::get_credential(&app_handle, "client_id")?;
    let client_secret = Vault::get_credential(&app_handle, "client_secret")?;

    let client = BasicClient::new(
        ClientId::new(client_id.clone()),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new("https://discord.com/oauth2/authorize".to_string()).unwrap(),
        Some(TokenUrl::new("https://discord.com/api/v9/oauth2/token".to_string()).unwrap()),
    );

    let (pkce_ch, pkce_ver) = PkceCodeChallenge::new_random_sha256();
    let (tx, rx) = oneshot::channel::<String>();

    let (listener_socket, port) = { // Renamed listener to listener_socket to avoid confusion
        let mut socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
        socket.set_reuse_address(true)?;
        #[cfg(not(windows))]
        socket.set_reuse_port(true)?;

        // Bind to port 0 to let the OS assign an ephemeral port
        let initial_addr = "127.0.0.1:0".parse::<SocketAddr>().map_err(|e| AppError {
            user_message: "Invalid bind address for dynamic port.".into(),
            technical_details: Some(e.to_string()),
            ..Default::default()
        })?;
        socket.bind(&initial_addr.into())?;
        socket.listen(128)?;

        let bound_addr = socket.local_addr()?.as_socket().expect("Failed to get socket address");
        let assigned_port = bound_addr.port();

        Logger::debug(
            &app_handle,
            &format!("[OAuth] Bound callback to 127.0.0.1:{}", assigned_port),
            None,
        );
        (socket.into(), assigned_port) // This socket.into() is the std::net::TcpListener
    };

    let client =
        client.set_redirect_uri(RedirectUrl::new(format!("http://127.0.0.1:{}", port)).unwrap());
    let (auth_url, csrf) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(oauth2::Scope::new("identify".into()))
        .add_scope(oauth2::Scope::new("guilds".into()))
        .set_pkce_challenge(pkce_ch)
        .url();

    let csrf_secret = csrf.secret().clone(); // Moved this line here
    let app_clone = app_handle.clone();

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
