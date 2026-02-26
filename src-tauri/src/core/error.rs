// src-tauri/src/core/error.rs

#[derive(serde::Serialize, Debug, Default)]
pub struct AppError {
    pub user_message: String,
    pub error_code: String,
    pub technical_details: Option<String>,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}): {:?}",
            self.user_message, self.error_code, self.technical_details
        )
    }
}

impl AppError {
    pub fn new(user_message: &str, error_code: &str) -> Self {
        Self {
            user_message: user_message.to_string(),
            error_code: error_code.to_string(),
            technical_details: None,
        }
    }

    pub fn from_discord_json(json: &serde_json::Value) -> Self {
        let code = json["code"].as_u64().unwrap_or(0);
        let message = json["message"]
            .as_str()
            .unwrap_or("Unknown Discord API error");

        let user_message = match code {
            50001 => "Insufficient Permissions: Access denied to this node.".to_string(),
            10003 => "Unknown Channel: Target node is unreachable.".to_string(),
            40002 => "Account Restricted: Verification or lockout active.".to_string(),
            50013 => "Missing Permissions: Higher elevation required.".to_string(),
            _ => format!("Discord Error: {}", message),
        };

        Self {
            user_message,
            error_code: format!("discord_api_{}", code),
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
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        Self {
            user_message: "Data parsing failed.".into(),
            error_code: "parse_error".into(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self {
            user_message: "System I/O failure.".into(),
            error_code: "io_error".into(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        Self {
            user_message: s,
            error_code: "internal_error".into(),
            technical_details: None,
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
        }
    }
}

impl From<tauri_plugin_opener::Error> for AppError {
    fn from(e: tauri_plugin_opener::Error) -> Self {
        Self {
            user_message: "Failed to open external resource.".into(),
            error_code: "opener_error".into(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for AppError {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        Self {
            user_message: "WebSocket communication error.".into(),
            error_code: "ws_error".into(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl From<tokio::time::error::Elapsed> for AppError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        Self {
            user_message: "Operation timed out.".into(),
            error_code: "timeout_error".into(),
            technical_details: None,
        }
    }
}

impl From<base64::DecodeError> for AppError {
    fn from(e: base64::DecodeError) -> Self {
        Self {
            user_message: "Data decoding failed.".into(),
            error_code: "decode_error".into(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl From<std::string::FromUtf8Error> for AppError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self {
            user_message: "Text encoding failed.".into(),
            error_code: "utf8_error".into(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl From<keyring::Error> for AppError {
    fn from(e: keyring::Error) -> Self {
        Self {
            user_message: "Secure storage access failed.".into(),
            error_code: "keyring_error".into(),
            technical_details: Some(e.to_string()),
        }
    }
}
