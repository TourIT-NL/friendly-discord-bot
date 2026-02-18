// src-tauri/src/core/error.rs

#[derive(serde::Serialize, Debug)] // Added Debug for better error reporting during development
pub struct AppError {
    /// A user-friendly message explaining what happened.
    pub user_message: String,
    /// A unique code for specific error types (e.g., 'discord_api_error', 'network_failure').
    pub error_code: String,
    /// Detailed technical information for logging.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technical_details: Option<String>,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}): {:?}", self.user_message, self.error_code, self.technical_details)
    }
}

impl std::error::Error for AppError {}

// --- Implement From for common error types ---

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self {
            user_message: "An internal I/O error occurred.".to_string(),
            error_code: "io_error".to_string(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for AppError {
    fn from(e: tokio::sync::oneshot::error::RecvError) -> Self {
        Self {
            user_message: "An internal communication error occurred.".to_string(),
            error_code: "oneshot_recv_error".to_string(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        Self {
            user_message: "A network or HTTP error occurred.".to_string(),
            error_code: "network_error".to_string(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl From<keyring::Error> for AppError {
    fn from(e: keyring::Error) -> Self {
        Self {
            user_message: "An error occurred while accessing the secure credential store.".to_string(),
            error_code: "keyring_error".to_string(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl From<tokio::task::JoinError> for AppError {
    fn from(e: tokio::task::JoinError) -> Self {
        Self {
            user_message: "An internal task error occurred.".to_string(),
            error_code: "task_join_error".to_string(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl From<tauri::Error> for AppError {
    fn from(e: tauri::Error) -> Self {
        Self {
            user_message: "An internal application error occurred.".to_string(),
            error_code: "tauri_error".to_string(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl From<tauri_plugin_opener::Error> for AppError {
    fn from(e: tauri_plugin_opener::Error) -> Self {
        Self {
            user_message: "An error occurred while opening an external application or URL.".to_string(),
            error_code: "opener_error".to_string(),
            technical_details: Some(e.to_string()),
        }
    }
}

impl From<oauth2::RequestTokenError<oauth2::reqwest::Error<reqwest::Error>, oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>>> for AppError {
    fn from(e: oauth2::RequestTokenError<oauth2::reqwest::Error<reqwest::Error>, oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>>) -> Self {
        Self {
            user_message: "Failed to exchange authorization code for tokens.".to_string(),
            error_code: "oauth_token_exchange_failure".to_string(),
            technical_details: Some(e.to_string()),
        }
    }
}



