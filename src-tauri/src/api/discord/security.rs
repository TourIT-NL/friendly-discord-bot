// src-tauri/src/api/discord/security.rs

use crate::api::rate_limiter::ApiHandle;
use crate::core::error::AppError;
use crate::core::forensics::auditor::IntegrationAuditor;
use crate::core::logger::Logger;
use crate::core::vault::Vault;
use reqwest::Method;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn fetch_oauth_tokens(app_handle: AppHandle) -> Result<serde_json::Value, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();

    Logger::info(
        &app_handle,
        "[SECURITY] Auditing third-party OAuth access",
        None,
    );

    let json = api_handle
        .send_request_json(
            Method::GET,
            "https://discord.com/api/v9/oauth2/tokens",
            None,
            &token,
            is_bearer,
            None,
        )
        .await?;

    // Forensic Risk Assessment
    let mut audited_apps = Vec::new();
    if let Some(apps) = json.as_array() {
        for app_json in apps {
            let risk_report = IntegrationAuditor::audit_app(app_json);
            let mut app_with_risk = app_json.clone();
            if let Some(obj) = app_with_risk.as_object_mut() {
                obj.insert(
                    "risk_report".to_string(),
                    serde_json::to_value(risk_report).unwrap(),
                );
            }
            audited_apps.push(app_with_risk);
        }
    }

    Ok(serde_json::to_value(audited_apps).unwrap())
}

#[tauri::command]
pub async fn revoke_oauth_token(app_handle: AppHandle, token_id: String) -> Result<(), AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();

    let _ = api_handle
        .send_request_json(
            Method::DELETE,
            &format!("https://discord.com/api/v9/oauth2/tokens/{}", token_id),
            None,
            &token,
            is_bearer,
            None,
        )
        .await?;

    Ok(())
}

#[tauri::command]
pub async fn fetch_application_identities(
    app_handle: AppHandle,
) -> Result<serde_json::Value, AppError> {
    let (token, is_bearer) = Vault::get_active_token(&app_handle)?;
    let api_handle = app_handle.state::<ApiHandle>();

    api_handle
        .send_request_json(
            Method::GET,
            "https://discord.com/api/v9/users/@me/application-identities",
            None,
            &token,
            is_bearer,
            None,
        )
        .await
}
