// src-tauri/src/core/error.rs

use crate::auth::types::DiscordError;

#[derive(serde::Serialize, Debug, Default)]
pub struct AppError {
    pub user_message: String,
    pub error_code: String,
    pub discord_code: Option<u32>,
    pub semantic_error: Option<DiscordError>,
    pub technical_details: Option<String>,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({:?}): {:?}",
            self.user_message, self.semantic_error, self.technical_details
        )
    }
}

impl AppError {
    pub fn new(user_message: &str, error_code: &str) -> Self {
        Self {
            user_message: user_message.to_string(),
            error_code: error_code.to_string(),
            discord_code: None,
            semantic_error: None,
            technical_details: None,
        }
    }

    pub fn from_discord_json(json: &serde_json::Value) -> Self {
        let code = json["code"].as_u64().unwrap_or(0) as u32;
        let message = json["message"]
            .as_str()
            .unwrap_or("Unknown Discord API error");

        let semantic = DiscordError::from_code(code);
        let user_message = match semantic {
            DiscordError::MissingAccess => {
                "Insufficient Permissions: Access denied to this node.".to_string()
            }
            DiscordError::UnknownChannel => {
                "Unknown Channel: Target node is unreachable.".to_string()
            }
            DiscordError::Unauthorized => {
                "Authentication Expired: Please log in again.".to_string()
            }
            DiscordError::RateLimited => "Rate Limited: Engine is cooling down.".to_string(),
            _ => format!("Discord Error: {}", message),
        };

        Self {
            user_message,
            error_code: format!("discord_api_{}", code),
            discord_code: Some(code),
            semantic_error: Some(semantic),
            technical_details: Some(json.to_string()),
        }
    }
}

impl std::error::Error for AppError {}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        Self {
            user_message: "Network request failed.".into(),
            error_code: "network_error".into(),
            technical_details: Some(e.to_string()),
            ..Default::default()
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        Self {
            user_message: "Data parsing failed.".into(),
            error_code: "parse_error".into(),
            technical_details: Some(e.to_string()),
            ..Default::default()
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self {
            user_message: "System I/O failure.".into(),
            error_code: "io_error".into(),
            technical_details: Some(e.to_string()),
            ..Default::default()
        }
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        Self {
            user_message: s,
            error_code: "internal_error".into(),
            ..Default::default()
        }
    }
}

impl From<oauth2::basic::BasicRequestTokenError<oauth2::reqwest::Error<reqwest::Error>>>
    for AppError
{
    fn from(
        e: oauth2::basic::BasicRequestTokenError<oauth2::reqwest::Error<reqwest::Error>>,
    ) -> Self {
        Self {
            user_message: "Failed to exchange authorization code.".into(),
            error_code: "oauth_error".into(),
            technical_details: Some(e.to_string()),
            ..Default::default()
        }
    }
}

impl From<tauri_plugin_opener::Error> for AppError {
    fn from(e: tauri_plugin_opener::Error) -> Self {
        Self {
            user_message: "Failed to open external resource.".into(),
            error_code: "opener_error".into(),
            technical_details: Some(e.to_string()),
            ..Default::default()
        }
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for AppError {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        Self {
            user_message: "WebSocket communication error.".into(),
            error_code: "ws_error".into(),
            technical_details: Some(e.to_string()),
            ..Default::default()
        }
    }
}

impl From<tokio::time::error::Elapsed> for AppError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        Self {
            user_message: "Operation timed out.".into(),
            error_code: "timeout_error".into(),
            ..Default::default()
        }
    }
}

impl From<base64::DecodeError> for AppError {
    fn from(e: base64::DecodeError) -> Self {
        Self {
            user_message: "Data decoding failed.".into(),
            error_code: "decode_error".into(),
            technical_details: Some(e.to_string()),
            ..Default::default()
        }
    }
}

impl From<std::string::FromUtf8Error> for AppError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self {
            user_message: "Text encoding failed.".into(),
            error_code: "utf8_error".into(),
            technical_details: Some(e.to_string()),
            ..Default::default()
        }
    }
}

impl From<keyring::Error> for AppError {
    fn from(e: keyring::Error) -> Self {
        Self {
            user_message: "Secure storage access failed.".into(),
            error_code: "keyring_error".into(),
            technical_details: Some(e.to_string()),
            ..Default::default()
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        Self {
            user_message: "Forensic cache failure.".into(),
            error_code: "database_error".into(),
            technical_details: Some(e.to_string()),
            ..Default::default()
        }
    }
}

impl From<argon2::Error> for AppError {
    fn from(e: argon2::Error) -> Self {
        Self {
            user_message: "Vault security engine failure.".into(),
            error_code: "argon2_error".into(),
            technical_details: Some(e.to_string()),
            ..Default::default()
        }
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(e: argon2::password_hash::Error) -> Self {
        Self {
            user_message: "Master password protocol failure.".into(),
            error_code: "password_hash_error".into(),
            technical_details: Some(e.to_string()),
            ..Default::default()
        }
    }
}

impl From<zip::result::ZipError> for AppError {
    fn from(e: zip::result::ZipError) -> Self {
        Self {
            user_message: "Archive processing failed.".into(),
            error_code: "zip_error".into(),
            technical_details: Some(e.to_string()),
            ..Default::default()
        }
    }
}
