# Discord Purge Error Handling Protocol: Ensuring Robustness and Clear User Feedback

This document outlines the standardized **error handling protocol** within the **Discord Purge utility**. Robust error management is critical for any **desktop application**, especially one that interacts with external APIs for sensitive operations like **Discord message deletion**. This protocol ensures consistent communication of errors from the Rust backend to the TypeScript frontend, providing clear user feedback and aiding in efficient debugging of the **Discord cleanup tool**.

Communication of errors from the **Rust backend** to the **TypeScript frontend** will adhere to a consistent and standardized `AppError` format, facilitating predictable error responses within the **Tauri application**.

```rust
// Scaffolding for src-tauri/src/core/error.rs
// Defines a serializable structure for application-specific errors in Discord Purge.
#[derive(serde::Serialize)]
pub struct AppError {
    /// A user-friendly message explaining what happened, designed for clear presentation in the Discord Purge UI.
    pub user_message: String,
    /// A unique code for specific error types (e.g., 'discord_api_error', 'network_failure', 'rate_limit_exceeded').
    /// This code helps the frontend categorize and respond to different issues in the Discord message management process.
    pub error_code: String,
    /// Detailed technical information for logging and developer troubleshooting. This field is optional
    /// and will be skipped during serialization if not present, preventing sensitive internal details
    /// from being exposed directly to the user interface.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technical_details: Option<String>,
}

// All Tauri commands within the Discord Purge application will consistently return a Result type.
// This allows for explicit error propagation and handling across the Rust-TypeScript boundary,
// ensuring the Discord privacy tool remains stable and responsive.
#[tauri::command]
async fn fetch_guilds() -> Result<Vec<Guild>, AppError> {
    // ... logic for fetching Discord guilds/servers, returning either a vector of Guilds
    // or an AppError if the operation fails (e.g., due to API issues, network problems).
}
```

This structured approach to **error communication** is vital for the stability and maintainability of the **Discord Purge application**, ensuring that users receive actionable feedback and developers can efficiently diagnose underlying issues related to **Discord data management**.
