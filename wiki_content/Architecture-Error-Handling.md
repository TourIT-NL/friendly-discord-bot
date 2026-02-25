# 🩺 Architecture: Error Handling & Resilience

Stability is a prerequisite for trust. This document details our robust, cross-platform error handling protocol.

---

## 🛠️ The Standardized Error Object

We use a unified `AppError` structure to bridge the gap between Rust's strong types and TypeScript's flexibility.

```rust
#[derive(serde::Serialize)]
pub struct AppError {
    /// Friendly message for the user interface.
    pub user_message: String,

    /// Unique internal code (e.g., 'discord_api_429').
    pub error_code: String,

    /// (Optional) Raw technical details for developer logs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technical_details: Option<String>,
}
```

---

## 🔄 The Handoff Logic

### 1. In Rust

Every Tauri command returns a `Result<T, AppError>`. If a low-level error occurs (like a network timeout), we catch it and wrap it in an `AppError` before it reaches the frontend.

### 2. In TypeScript

We use a centralized `handleApiError` hook. This ensures that every error:

- Is logged to the Developer Console with full context.
- Triggers an appropriate UI notification (Error Overlay).
- Corrects the loading state of the application.

---

## 🚨 Critical Error Categories

- **Vault Errors**: Occur when the OS Keychain access fails.
- **Discord API Errors**: Handled via our Rate Limiter Actor.
- **Authentication Errors**: Triggers a redirect to the Login screen.
- **Permission Errors**: Informs the user they need to grant specific scopes.

---

## 🛡️ Stability Safeguards

- **Boundary Catching**: Any panic in the Rust backend is captured to prevent the entire desktop window from crashing.
- **Graceful Degradation**: If one server channel fails to load, the rest of the application remains functional.

_Last updated: February 25, 2026_
