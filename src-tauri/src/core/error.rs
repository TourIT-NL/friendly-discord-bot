// src-tauri/src/core/error.rs

#[derive(serde::Serialize, Debug)]
pub struct AppError {
    pub user_message: String,
    pub error_code: String,
    pub technical_details: Option<String>,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}): {:?}", self.user_message, self.error_code, self.technical_details)
    }
}

impl std::error::Error for AppError {}

impl Default for AppError {
    fn default() -> Self {
        Self {
            user_message: "An internal system error occurred.".to_string(),
            error_code: "internal_error".to_string(),
            technical_details: None,
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self {
            user_message: "I/O failure.".into(),
            error_code: "io_error".into(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        Self {
            user_message: "Network request failed.".into(),
            error_code: "network_error".into(),
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

impl From<tauri::Error> for AppError {
    fn from(e: tauri::Error) -> Self {
        Self {
            user_message: "Application bridge error.".into(),
            error_code: "tauri_error".into(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl From<tauri_plugin_opener::Error> for AppError {
    fn from(e: tauri_plugin_opener::Error) -> Self {
        Self {
            user_message: "Failed to open external link.".into(),
            error_code: "opener_error".into(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl From<url::ParseError> for AppError {
    fn from(e: url::ParseError) -> Self {
        Self {
            user_message: "Invalid URL structure.".into(),
            error_code: "url_error".into(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for AppError {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        Self {
            user_message: "Secure connection failed.".into(),
            error_code: "websocket_error".into(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for AppError {
    fn from(e: tokio::sync::oneshot::error::RecvError) -> Self {
        Self {
            user_message: "Process communication timed out.".into(),
            error_code: "oneshot_error".into(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl From<tokio::time::error::Elapsed> for AppError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        Self {
            user_message: "The operation timed out.".into(),
            error_code: "timeout".into(),
            technical_details: None,
        }
    }
}

impl From<oauth2::RequestTokenError<oauth2::reqwest::Error<reqwest::Error>, oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>>> for AppError {
    fn from(e: oauth2::RequestTokenError<oauth2::reqwest::Error<reqwest::Error>, oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>>) -> Self {
        Self {
            user_message: "Failed to exchange authorization code.".into(),
            error_code: "oauth_error".into(),
            technical_details: Some(e.to_string()),
        }
    }
}
