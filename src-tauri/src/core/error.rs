// src-tauri/src/core/error.rs

#[derive(serde::Serialize, Debug)]
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
        let (error_code, user_message) = if e.is_timeout() {
            ("network_timeout".to_string(), "Network request timed out.".to_string())
        } else if e.is_connect() {
            ("network_connect_error".to_string(), "Failed to connect to network host.".to_string())
        } else if e.is_decode() {
            ("network_decode_error".to_string(), "Failed to decode network response.".to_string())
        } else if e.is_status() {
            (format!("network_http_{}", e.status().unwrap_or_default().as_u16()),
             format!("HTTP error: {}", e.status().unwrap_or_default()))
        } else if e.is_builder() {
            ("network_request_build_error".to_string(), "Failed to build network request.".to_string())
        } else {
            ("network_error".to_string(), "Network request failed.".to_string())
        };
        Self {
            user_message,
            error_code,
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
        match e {
            tauri::Error::Io(io_err) => AppError::from(io_err), // Delegate
            tauri::Error::Json(json_err) => AppError::from(json_err), // Delegate
            // For other tauri::Error variants, return a generic tauri_error
            // or match on specific known ones if needed.
            _ => Self {
                user_message: "Application bridge error.".into(),
                error_code: "tauri_error".into(),
                technical_details: Some(e.to_string()),
            },
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

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        Self {
            user_message: "Data parsing error.".into(),
            error_code: "json_error".into(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl From<base64::DecodeError> for AppError {
    fn from(e: base64::DecodeError) -> Self {
        Self {
            user_message: "Data encoding/decoding error.".into(),
            error_code: "encoding_error".into(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl From<std::string::FromUtf8Error> for AppError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self {
            user_message: "Invalid UTF-8 sequence.".into(),
            error_code: "utf8_error".into(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl
    From<
        oauth2::RequestTokenError<
            oauth2::reqwest::Error<reqwest::Error>,
            oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        >,
    > for AppError
{
    fn from(
        e: oauth2::RequestTokenError<
            oauth2::reqwest::Error<reqwest::Error>,
            oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        >,
    ) -> Self {
        Self {
            user_message: "Failed to exchange authorization code.".into(),
            error_code: "oauth_error".into(),
            technical_details: Some(e.to_string()),
        }
    }
}
