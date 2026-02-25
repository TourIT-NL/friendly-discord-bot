# 🩺 Architecture: Error Handling & Resilience

Stability is a prerequisite for user trust. This document details our robust, cross-platform error handling protocol, designed to ensure **Discord Purge** never crashes silently and always provides actionable feedback.

---

## 🛠️ The Unified Error Model

We bridge the gap between Rust's strict types and TypeScript's flexibility using a standardized `AppError` object.

```rust
#[derive(serde::Serialize, Debug)]
pub struct AppError {
    /// A human-readable message for the UI (translated/localized in the future).
    pub user_message: String,

    /// A machine-readable code for logic branching (e.g., 'auth_failed_token_invalid').
    pub error_code: String,

    /// (Optional) Internal technical details, excluded from production logs for privacy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technical_details: Option<String>,
}
```

---

## 🔄 The Handoff Logic

Our communication follows a predictable "Catch-Wrap-Emit" pattern.

### 1. In Rust (The Producer)

Every Tauri command returns a `Result<T, AppError>`.

- **Internal Errors**: We use the `?` operator to bubble up errors.
- **Conversion**: We implement `From<InternalError> for AppError` to ensure seamless conversion.
- **Security**: We sanitize errors to ensure sensitive system paths or API keys aren't leaked in the message.

### 2. In TypeScript (The Consumer)

We use a centralized `handleApiError` utility hook.

- **Consolidation**: It updates the `authStore` with the latest error state.
- **Notification**: It triggers the **Global Error Overlay**.
- **Persistence**: Critical errors are saved to the **Local Log File** for later review.

---

## 🚨 Error Categories & Responses

| Category       | Typical Cause                        | App Response                                                        |
| :------------- | :----------------------------------- | :------------------------------------------------------------------ |
| **Vault**      | OS Keychain locked or access denied. | Redirect to "Setup" view to re-initialize credentials.              |
| **Rate Limit** | Too many concurrent deletions.       | Backend Actor pauses automatically; UI shows "Waiting".             |
| **Session**    | Access token expired.                | Automatic attempt to refresh; if failed, redirect to Login.         |
| **Network**    | Offline or Discord API outage.       | Retry with exponential backoff; notify user of "Connection Issues". |

---

## 🛡️ Stability Safeguards

- **Boundary Protection**: Any panic in the backend is captured by Tauri's wrapper, preventing the entire OS process from exiting.
- **Graceful Degradation**: If one server's data fails to load, we isolate that failure and allow the user to continue cleaning up other servers.

_Last updated: February 25, 2026_
