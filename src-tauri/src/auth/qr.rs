// src-tauri/src/auth/qr.rs

use base64::{Engine as _, engine::general_purpose};
use futures_util::{SinkExt, StreamExt};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey, pkcs8::EncodePublicKey}; // Updated import
use tauri::{AppHandle, Emitter, Window};
use tokio::time::{Duration, timeout};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_util::sync::CancellationToken;
use sha2::{Sha256, Digest};

use super::identity::login_with_token_internal;
use super::types::AuthState;
use crate::core::error::AppError;
use crate::core::logger::Logger;
use crate::core::vault::Vault; // NEW

#[tauri::command]
pub async fn start_qr_login_flow(
    app_handle: AppHandle,
    window: Window,
    state: tauri::State<'_, AuthState>,
) -> Result<(), AppError> {
    Logger::info(&app_handle, "[QR] Initializing secure handshake...", None);

    // Check for client credentials first
    let client_id = match Vault::get_credential(&app_handle, "client_id") {
        Ok(id) => id,
        Err(e) if e.error_code == "vault_credentials_missing" => return Err(e),
        Err(e) => return Err(e),
    };
    let _client_secret = match Vault::get_credential(&app_handle, "client_secret") {
        Ok(secret) => secret,
        Err(e) if e.error_code == "vault_credentials_missing" => return Err(e),
        Err(e) => return Err(e),
    };

    // Generate RSA Keypair (ensure rng is not held across await)
    let priv_key = {
        let mut rng = rand::thread_rng();
        RsaPrivateKey::new(&mut rng, 2048).map_err(|_| AppError {
            user_message: "RSA generation failed.".into(),
            ..Default::default()
        })?
    };
    let pub_key = RsaPublicKey::from(&priv_key);

    let pub_key_der = pub_key
        .to_public_key_der() // Use SPKI (pkcs8) instead of pkcs1
        .map_err(|_| AppError {
            user_message: "Public key encoding failed.".into(),
            ..Default::default()
        })?;
    let pub_key_base64 = general_purpose::STANDARD.encode(pub_key_der.as_bytes());

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
    request
        .headers_mut()
        .insert("Origin", "https://discord.com".parse().unwrap());

    Logger::debug(&app_handle, "[QR] Establishing WebSocket link", None);
    let ws_connect_result = timeout(Duration::from_secs(10), connect_async(request)).await;

    let (ws_stream, _) = match ws_connect_result {
        Ok(Ok(stream_pair)) => {
            Logger::info(&app_handle, "[QR] WebSocket connected successfully.", None);
            stream_pair
        },
        Ok(Err(e)) => {
            Logger::error(&app_handle, &format!("[QR] WebSocket connection failed: {:?}", e), None);
            return Err(AppError::from(e));
        },
        Err(e) => {
            Logger::error(&app_handle, &format!("[QR] WebSocket connection timed out: {:?}", e), None);
            return Err(AppError::from(e));
        },
    };
    
    let (mut write, mut read) = ws_stream.split();

    let window_clone = window.clone();
    let app_handle_clone = app_handle.clone();

    tauri::async_runtime::spawn(async move {
        let mut interval_ms = 30000;
        let mut heartbeat_interval = tokio::time::interval(Duration::from_millis(interval_ms));

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    Logger::info(&app_handle_clone, "[QR] Session aborted by user", None);
                    break;
                }
                _ = heartbeat_interval.tick() => {
                    let _ = write.send(Message::Text(serde_json::json!({"op": "heartbeat"}).to_string().into())).await;
                }
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&text) {
                                let op = p["op"].as_str().unwrap_or("unknown");
                                Logger::trace(&app_handle_clone, &format!("[QR] Received Opcode: {} | Payload: {:?}", op, p), None);
                                
                                match op {
                                    "hello" => {
                                        interval_ms = p["heartbeat_interval"].as_u64().unwrap_or(30000);
                                        heartbeat_interval = tokio::time::interval(Duration::from_millis(interval_ms));
                                        heartbeat_interval.tick().await; 
                                        
                                        let init_payload = serde_json::json!({
                                            "op": "init",
                                            "encoded_public_key": pub_key_base64,
                                            "client_id": client_id // Include client_id here
                                        });
                                        Logger::debug(&app_handle_clone, "[QR] Sending INIT payload.", None);
                                        let _ = write.send(Message::Text(init_payload.to_string().into())).await;
                                        Logger::debug(&app_handle_clone, "[QR] Secure handshake initiated", None);
                                    },
                                    "nonce_proof" => {
                                        Logger::debug(&app_handle_clone, "[QR] Received nonce proof request", None);
                                        let encrypted_nonce = p["encrypted_nonce"].as_str().unwrap_or_default();
                                        match general_purpose::STANDARD.decode(encrypted_nonce) {
                                            Ok(encrypted_bytes) => {
                                                match priv_key.decrypt(Pkcs1v15Encrypt, &encrypted_bytes) {
                                                    Ok(decrypted) => {
                                                        let mut hasher = Sha256::new();
                                                        hasher.update(&decrypted);
                                                        let hash = hasher.finalize();
                                                        let proof = general_purpose::URL_SAFE_NO_PAD.encode(hash);
                                                        let _ = write.send(Message::Text(serde_json::json!({"op": "nonce_proof", "proof": proof}).to_string().into())).await;
                                                        Logger::debug(&app_handle_clone, "[QR] Sent nonce proof response", None);
                                                    },
                                                    Err(e) => Logger::error(&app_handle_clone, "[QR] Nonce decryption failed", Some(serde_json::json!({"error": e.to_string()}))),
                                                }
                                            },
                                            Err(e) => Logger::error(&app_handle_clone, "[QR] Nonce base64 decode failed", Some(serde_json::json!({"error": e.to_string()}))),
                                        }
                                    },
                                    "fingerprint" => {
                                        if let Some(fp) = p["fingerprint"].as_str() {
                                            Logger::info(&app_handle_clone, "[QR] Signature generated", None);
                                            let _ = window_clone.emit("qr_code_ready", format!("https://discord.com/ra/{}", fp));
                                        }
                                    },
                                    "pending_remote_init" => {
                                        Logger::info(&app_handle_clone, "[QR] Remote scan detected. Awaiting confirmation...", None);
                                        let _ = window_clone.emit("qr_scanned", ());
                                    },
                                    "finish" => {
                                        Logger::info(&app_handle_clone, "[QR] Handshake finalized", None);
                                        let encrypted_token = p["encrypted_token"].as_str().unwrap_or_default();
                                        match general_purpose::STANDARD.decode(encrypted_token) {
                                            Ok(encrypted_bytes) => {
                                                match priv_key.decrypt(Pkcs1v15Encrypt, &encrypted_bytes) {
                                                    Ok(decrypted) => {
                                                        let token = String::from_utf8_lossy(&decrypted).to_string();
                                                        let _ = login_with_token_internal(app_handle_clone.clone(), window_clone.clone(), token, false).await;
                                                    },
                                                    Err(e) => Logger::error(&app_handle_clone, "[QR] Token decryption failed", Some(serde_json::json!({"error": e.to_string()}))),
                                                }
                                            },
                                            Err(e) => Logger::error(&app_handle_clone, "[QR] Token base64 decode failed", Some(serde_json::json!({"error": e.to_string()}))),
                                        }
                                        break;
                                    },
                                    _ => {}
                                }
                            }
                        },
                        Some(Ok(Message::Close(frame))) => {
                            Logger::warn(&app_handle_clone, &format!("[QR] WebSocket closed by server: {:?}", frame), None);
                            break;
                        },
                        Some(Err(e)) => {
                            Logger::error(&app_handle_clone, &format!("[QR] WebSocket error: {:?}", e), None);
                            break;
                        },
                        None => {
                            Logger::warn(&app_handle_clone, "[QR] WebSocket stream ended unexpectedly (None)", None);
                            break;
                        },
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
