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
use tracing::{info, warn, error};

// Placeholder for client ID and secret - these will need to be configured
// Client ID and secret will be read from environment variables or keyring at runtime
fn get_discord_client_id() -> Result<String, AppError> {
    if let Ok(id) = std::env::var("DISCORD_CLIENT_ID") {
        return Ok(id);
    }
    
    // Check keyring as fallback
    let entry = Entry::new("discord_privacy_util", "client_id")
        .map_err(AppError::from)?;
    
    entry.get_password().map_err(|e| {
        error!("Discord Client ID not set in environment or keyring: {}", e);
        AppError {
            user_message: "Discord Client ID not configured. Please enter it in the settings.".to_string(),
            error_code: "credentials_missing".to_string(),
            technical_details: Some(e.to_string()),
        }
    })
}

fn get_discord_client_secret() -> Result<String, AppError> {
    if let Ok(secret) = std::env::var("DISCORD_CLIENT_SECRET") {
        return Ok(secret);
    }

    // Check keyring as fallback
    let entry = Entry::new("discord_privacy_util", "client_secret")
        .map_err(AppError::from)?;

    entry.get_password().map_err(|e| {
        error!("Discord Client Secret not set in environment or keyring: {}", e);
        AppError {
            user_message: "Discord Client Secret not configured. Please enter it in the settings.".to_string(),
            error_code: "credentials_missing".to_string(),
            technical_details: Some(e.to_string()),
        }
    })
}

#[tauri::command]
pub async fn save_discord_credentials(client_id: String, client_secret: String) -> Result<(), AppError> {
    info!("Saving Discord API credentials to secure keyring...");
    
    let id_entry = Entry::new("discord_privacy_util", "client_id")
        .map_err(AppError::from)?;
    id_entry.set_password(&client_id).map_err(AppError::from)?;

    let secret_entry = Entry::new("discord_privacy_util", "client_secret")
        .map_err(AppError::from)?;
    secret_entry.set_password(&client_secret).map_err(AppError::from)?;

    info!("Credentials saved successfully.");
    Ok(())
}
const DISCORD_REDIRECT_PATH: &str = "/auth/callback";


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub avatar: Option<String>,
    pub email: Option<String>,
}

#[tauri::command]
pub async fn start_oauth_flow(app_handle: AppHandle, window: Window) -> Result<DiscordUser, AppError> {
    info!("Starting OAuth2 flow with Discord...");
    // 1. Setup the OAuth2 client
    let client = BasicClient::new(
        ClientId::new(get_discord_client_id()?),
        Some(ClientSecret::new(get_discord_client_secret()?)),
        AuthUrl::new("https://discord.com/oauth2/authorize".to_string())
            .map_err(|e: oauth2::url::ParseError| AppError {
                user_message: "Failed to create Discord authorization URL.".to_string(),
                error_code: "invalid_auth_url".to_string(),
                technical_details: Some(e.to_string()),
            })?,
        Some(TokenUrl::new("https://discord.com/api/oauth2/token".to_string())
            .map_err(|e: oauth2::url::ParseError| AppError {
                user_message: "Failed to create Discord token URL.".to_string(),
                error_code: "invalid_token_url".to_string(),
                technical_details: Some(e.to_string()),
            })?),
    );


    // 2. Generate PKCE challenge and verifier
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    // 3. Generate a random state to prevent CSRF
    let csrf_state = CsrfToken::new_random();
    
    // 4. Start a temporary local server to listen for the Discord callback
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e: std::io::Error| {
            error!("Failed to bind local TCP listener for OAuth: {}", e);
            AppError {
                user_message: "Failed to start local server for OAuth callback.".to_string(),
                error_code: "tcp_bind_failure".to_string(),
                technical_details: Some(e.to_string()),
            }
        })?;
    let port = listener.local_addr()
        .map_err(|e: std::io::Error| {
            error!("Failed to get local listener address: {}", e);
            AppError {
                user_message: "Failed to get local server address.".to_string(),
                error_code: "tcp_addr_failure".to_string(),
                technical_details: Some(e.to_string()),
            }
        })?
        .port();

    info!("OAuth callback server listening on port {}", port);

    let redirect_url = RedirectUrl::new(format!("http://127.0.0.1:{}{}", port, DISCORD_REDIRECT_PATH))
        .map_err(|e: oauth2::url::ParseError| AppError {
            user_message: "Failed to construct redirect URL for OAuth callback.".to_string(),
            error_code: "invalid_redirect_url".to_string(),
            technical_details: Some(e.to_string()),
        })?;

    let client = client.set_redirect_uri(redirect_url.clone());

    let (authorize_url, _) = client
        .authorize_url(|| csrf_state.clone()) // Pass closure for CSRF token
        .add_scope(oauth2::Scope::new("identify".to_string()))
        .add_scope(oauth2::Scope::new("guilds".to_string()))
        .add_scope(oauth2::Scope::new("email".to_string()))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    // Store csrf_state and pkce_code_verifier to validate later
    let csrf_state_str = Arc::new(StdMutex::new(csrf_state.secret().to_string()));
    let pkce_code_verifier_str = Arc::new(StdMutex::new(pkce_code_verifier.secret().to_string()));


    let (tx_code, rx_code) = oneshot::channel(); // Channel to receive auth code from web server
    let (tx_error, rx_error) = oneshot::channel(); // Channel to propagate errors from web server
    
    let server_handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::from_std(listener)
            .map_err(|e: std::io::Error| {
                error!("Failed to convert standard TcpListener to Tokio: {}", e);
                AppError {
                    user_message: "Failed to convert standard TcpListener to Tokio's async version.".to_string(),
                    error_code: "tcp_listener_conversion_failure".to_string(),
                    technical_details: Some(e.to_string()),
                }
            })?;
        
        let (mut stream, _): (tokio::net::TcpStream, std::net::SocketAddr) = listener.accept().await?;
        
        let mut buffer = [0; 2048];
        let n = stream.read(&mut buffer).await?;
        let request = String::from_utf8_lossy(&buffer[..n]);

        let request_uri = request.split_whitespace().nth(1).ok_or_else(|| {
            error!("Received malformed request URI during OAuth callback.");
            AppError {
                user_message: "Malformed request URI received during OAuth callback.".to_string(),
                error_code: "malformed_request_uri".to_string(),
                technical_details: None,
            }
        })?;
        let query_params: HashMap<String, String> = Url::parse(&format!("http://localhost{}", request_uri))
            .map_err(|e: oauth2::url::ParseError| AppError {
                user_message: "Failed to parse request URI from OAuth callback.".to_string(),
                error_code: "uri_parse_failure".to_string(),
                technical_details: Some(e.to_string()),
            })?
            .query_pairs()
            .map(|(k, v): (std::borrow::Cow<'_, str>, std::borrow::Cow<'_, str>)| (k.into_owned(), v.into_owned()))
            .collect();

        let code = query_params.get("code").ok_or_else(|| {
            warn!("Authorization code missing from Discord callback.");
            AppError {
                user_message: "Authorization code not found in OAuth callback.".to_string(),
                error_code: "auth_code_missing".to_string(),
                technical_details: None,
            }
        })?.to_string();
        let received_state = query_params.get("state").ok_or_else(|| {
            warn!("State parameter missing from Discord callback.");
            AppError {
                user_message: "State parameter not found in OAuth callback.".to_string(),
                error_code: "state_missing".to_string(),
                technical_details: None,
            }
        })?.to_string();

        let expected_csrf_state = csrf_state_str.lock().unwrap().clone();
        if received_state != expected_csrf_state {
            error!("CSRF state mismatch: expected {}, received {}", expected_csrf_state, received_state);
            let _ = tx_error.send(AppError {
                user_message: "OAuth state mismatch detected. Possible CSRF attack or invalid callback.".to_string(),
                error_code: "csrf_state_mismatch".to_string(),
                technical_details: None,
            });
            return Err(AppError {
                user_message: "OAuth state mismatch detected. Possible CSRF attack or invalid callback.".to_string(),
                error_code: "csrf_state_mismatch".to_string(),
                technical_details: None,
            });
        }

        // Send a minimal HTTP response to the browser
        let response_body = "Authentication successful! You can now close this tab.";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response_body.len(),
            response_body
        );
        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;

        let _ = tx_code.send((code, pkce_code_verifier_str.lock().unwrap().clone()));
        Ok(())
    });

    // 5. Open the URL in the user's default browser
    window.emit("auth_started", ())
        .map_err(|e: tauri::Error| AppError::from(e))?;

    app_handle.opener().open_url(authorize_url.to_string(), None::<&str>)
        .map_err(|e: tauri_plugin_opener::Error| AppError {
            user_message: "Failed to open Discord authorization URL in browser.".to_string(),
            error_code: "browser_open_failure".to_string(),
            technical_details: Some(e.to_string()),
        })?;

    let (auth_code, pkce_code_verifier): (String, String) = tokio::select! {
        code_result = rx_code => code_result?,
        error_result = rx_error => {
            return Err(error_result?);
        }
    };
    
    // Ensure the server handling task doesn't just get dropped
    let _ = server_handle.await?;

    info!("Received authorization code. Exchanging for tokens...");

    // 6. Exchange the authorization code for an Access Token and Refresh Token
    let token_response = client
        .exchange_code(oauth2::AuthorizationCode::new(auth_code))
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_code_verifier))
        .request_async(oauth2::reqwest::async_http_client)
        .await?;

    let access_token = token_response.access_token().secret().to_string();
    let refresh_token = token_response.refresh_token().map(|t: &oauth2::RefreshToken| t.secret().to_string());

    info!("Token exchange successful. Storing credentials...");

    // 7. Securely store tokens using keyring
    let entry = Entry::new("discord_privacy_util", "discord_user")?;
    
    entry.set_password(&format!("ACCESS_TOKEN={}\nREFRESH_TOKEN={}", access_token, refresh_token.unwrap_or_default()))?;

    // 8. Fetch user profile from Discord
    info!("Fetching user profile from Discord...");
    let api_handle = app_handle.state::<ApiHandle>();
    let response = api_handle.send_request(
        reqwest::Method::GET,
        "https://discord.com/api/users/@me",
        None,
        &access_token
    ).await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        error!("Failed to fetch user profile: {} - {}", status, body);
        return Err(AppError {
            user_message: "Failed to fetch user profile from Discord.".to_string(),
            error_code: "user_profile_fetch_failure".to_string(),
            technical_details: Some(format!("Status: {}, Body: {}", status, body)),
        });
    }

    let user_profile: DiscordUser = response.json().await?;

    info!("Successfully logged in as {} (ID: {})", user_profile.username, user_profile.id);

    // 9. Emit a success event to the frontend
    window.emit("auth_success", user_profile.clone()).unwrap();

    Ok(user_profile)
}
