# Project: Discord Privacy and Cleanup Utility - The Final Master Blueprint

MANDATORY LAWS:
At the start of task; read all .rs and .json files in context, as well as all web files.
Setup/Configure/Verify a local GIT for this project.
We do everything through GIT, every change should be done through GIT
Document everything! Be as verbose as possible.
Always check context of files.
Try not to create monoliths, but streamlined seperate modules that can be called anywhere in the project.
Reasearch any external package or tool we use, and make sure we use the latest versions, and have an up to date documentation.
We don't use anything updated prior January 2026.
When an error is flagged, read all relevant files and their called files or dependancies prior to making any edits. Too many times do I see you recreate entire files, rather than fixing them inline or appending logic.

## 0. Document Purpose

This document is the **Final Master Blueprint** for the Discord Privacy and Cleanup Utility. It is to be considered the single, unambiguous, exhaustive, and binding specification for this project. All development work must adhere strictly to the principles, architectures, and workflows defined herein. No deviation is permitted without a formal amendment to this document.

## 1. Vision & Core Tenets

The project's vision is to create a powerful, aesthetically pleasing, and high-performance desktop application that empowers Discord users with ultimate control over their digital footprint.

Every decision must be measured against three core tenets:
1.  **Unyielding Security**: The user's data and account integrity are paramount. There will be no shortcuts. This includes both technical implementation (e.g., OAuth2, secure storage) and user experience design (e.g., clear warnings for destructive actions).
2.  **Peak Performance**: The application must feel fast and responsive, regardless of the scale of the task. Asynchronous operations and a compiled backend are not optional; they are fundamental requirements.
3.  **Total User Empowerment**: The application's purpose is to give control back to the user. This means clear, intuitive interfaces and powerful, reliable tools that perform as advertised.

## 2. User Stories & Acceptance Criteria

### 2.1. Authentication (US-001)
-   **User Story**: As a new user, I want to securely log in to the application using my Discord account so that I can access its features without ever exposing my password or token.
-   **Acceptance Criteria**:
    1.  The application must present a "Login with Discord" button on the initial screen.
    2.  Clicking the button must open my default web browser to the official Discord consent screen.
    3.  The requested permissions (scopes) must be clearly listed on the consent screen.
    4.  After approving, I am redirected to a page that confirms success and instructs me to return to the app.
    5.  The application window must automatically transition to the main authenticated interface.
    6.  On subsequent launches, the application must remember my session and log me in automatically.

### 2.2. Bulk Message Deletion (US-002)
-   **User Story**: As a privacy-conscious user, I want to permanently delete messages in bulk from specific channels, DMs, or group chats so that I can manage my chat history.
-   **Acceptance Criteria**:
    1.  I can view a list of all my servers, channels, and DM conversations.
    2.  I can select one or more of these locations for deletion.
    3.  I can select a time frame: "Last 24 Hours," "Last 7 Days," "All Time," or a custom date range.
    4.  Before starting, a final confirmation modal must appear, stating exactly what will be deleted (e.g., "This will permanently delete all messages from #general and 2 other channels.").
    5.  To proceed, I must type the word `DELETE` into a confirmation field within the modal.
    6.  During the process, a real-time progress bar and status text must show which channel is being processed and how many messages have been deleted.

### 2.3. Bulk Server Departure (US-003)
-   **User Story**: As a user decluttering my account, I want to leave multiple servers at once while staying in a select few.
-   **Acceptance Criteria**:
    1.  I can view a list of all servers I am a member of, with checkboxes next to each.
    2.  By default, all servers are checked.
    3.  I can uncheck servers to create a "whitelist" of servers to remain in.
    4.  The final confirmation modal requires me to type `LEAVE` to proceed.
    5.  The UI provides real-time feedback as it leaves each server.

## 3. Detailed Architecture & Implementation

### 3.1. OAuth2 Flow Diagram (Textual)
```
[React UI]          -> User clicks "Login"
[React UI]          -> invoke('start_oauth_flow')
[Rust Backend]      -> start_oauth_flow() is called
[Rust Backend]      -> Generates PKCE challenge & state
[Rust Backend]      -> Starts temporary local server on a random free port (e.g., 58123)
[Rust Backend]      -> Constructs Discord URL (with client_id, scopes, pkce, state)
[Rust Backend]      -> tauri::api::shell::open(URL)
----------------------------------------------------------------------------------
[User's Browser]    -> User logs in, sees consent screen, clicks "Authorize"
[User's Browser]    -> Discord redirects to http://localhost:58123/callback?code=...&state=...
----------------------------------------------------------------------------------
[Rust Backend]      -> Local server receives request, validates state parameter
[Rust Backend]      -> Shuts down local server immediately
[Rust Backend]      -> Exchanges authorization code (+ PKCE verifier) for tokens via POST to Discord API
[Rust Backend]      -> Stores access_token & refresh_token securely in OS keychain
[Rust Backend]      -> window.emit('auth_success', user_profile)
[React UI]          -> Listens for 'auth_success' event, updates Zustand store, redirects to main app view
```

### 3.2. The Rate Limiting Actor (Rust)
A dedicated, asynchronous actor is required to handle global rate limiting.
```rust
// Scaffolding for src-tauri/src/api/rate_limiter.rs

use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;
use std::time::Duration;

// Represents a pending API request
pub struct ApiRequest { /* ... fields for method, url, body ... */ }
pub struct ApiResponse { /* ... fields for status, body ... */ }

// The actor's state
pub struct RateLimiterActor {
    // A queue for incoming requests
    inbox: mpsc::Receiver<ApiRequest>,
    // Information about the current rate limit window
    rate_limit_info: Arc<Mutex<RateLimitInfo>>,
}

#[derive(Clone, Default)]
pub struct RateLimitInfo {
    remaining: u32,
    reset_after: Duration,
}

impl RateLimiterActor {
    pub async fn run(&mut self) {
        while let Some(request) = self.inbox.recv().await {
            let mut info = self.rate_limit_info.lock().await;
            if info.remaining == 0 {
                // If we've hit the limit, sleep until the window resets
                tokio::time::sleep(info.reset_after).await;
            }
            // ... execute the request with `reqwest` ...
            // ... parse X-RateLimit-* headers from the response ...
            // ... update info struct ...
            // ... send response back to the caller ...
        }
    }
}
```

### 3.3. Error Handling Protocol
Communication of errors from Rust to TypeScript will follow a standardized format.
```rust
// Scaffolding for src-tauri/src/core/error.rs
#[derive(serde::Serialize)]
pub struct AppError {
    /// A user-friendly message explaining what happened.
    pub user_message: String,
    /// A unique code for specific error types (e.g., 'discord_api_error', 'network_failure').
    pub error_code: String,
    /// Detailed technical information for logging.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technical_details: Option<String>,
}

// All Tauri commands will return Result<T, AppError>
#[tauri::command]
async fn fetch_guilds() -> Result<Vec<Guild>, AppError> {
    // ... logic ...
}
```

### 3.4. Account & GDPR Management Flow

Given Discord's API limitations, direct programmatic account deletion or bulk GDPR data deletion is not feasible. The utility will guide users to Discord's official processes.

*   **Multi-Server Message Deletion**: The backend `bulk_delete_messages` command already accepts a `Vec<String>` of channel IDs, meaning it can process channels from multiple guilds/DMs in a single operation. The challenge is primarily in the frontend UI to allow for multi-selection.
*   **Profile Deletion**:
    *   **Backend (`src-tauri/src/api/discord.rs`)**: A new Tauri command `open_discord_url_for_action(action_type: String)` will be implemented. This command will take an `action_type` (e.g., "account_deletion") and use `tauri::api::shell::open` to open the relevant Discord URL in the user's default browser (e.g., Discord's account settings page for deletion).
    *   **Frontend (`src/App.tsx`)**: A "Delete Profile" button will trigger a confirmation modal with strong warnings. Upon user confirmation, it will call the `open_discord_url_for_action` backend command, guiding the user to Discord's official account deletion process. The UI will then log out the user.
*   **GDPR Data Request**:
    *   **Backend**: The `open_discord_url_for_action` command can also be used to open Discord's Data & Privacy settings or support page.
    *   **Frontend**: A "GDPR Data Request" button (or similar UI element) will provide clear instructions:
        1.  How to download their data package from Discord's User Settings > Data & Privacy.
        2.  How to formally contact Discord Support to request bulk message deletion, possibly including information on providing a CSV of channel IDs (derived from their data package). This will likely involve opening Discord's support portal via `open_discord_url_for_action`.

## 4. Scaffolding & Project Structure

The directory structure and setup commands from the previous version are sound and will be adhered to.

## 5. Logging Strategy

-   **Library**: The `tracing` crate will be used in Rust for structured logging.
-   **Levels**: `INFO` for major lifecycle events (e.g., app start, login success), `WARN` for non-critical issues (e.g., a single API call fails but can be retried), `ERROR` for critical failures.
-   **Output**: Logs will be written to both the console (during development) and a rotating log file in the user's app data directory (e.g., `.../app.log`). The `tracing_appender` crate will be used for file rotation, keeping a maximum of 3 log files of 5MB each.

## 6. Testing Methodology

### 6.1. Rust Backend
-   **Unit Tests**: Pure functions (e.g., data transformation) will be tested in isolated unit tests.
    ```rust
    #[cfg(test)]
    mod tests {
        #[test]
        fn it_works() {
            assert_eq!(2 + 2, 4);
        }
    }
    ```
-   **Integration Tests**: The Discord API client will be tested via integration tests that use a mock HTTP server (e.g., `wiremock-rs`) to simulate Discord's API responses, including rate limit headers and error codes.

### 6.2. TypeScript Frontend
-   **Component Tests**: `Vitest` and `React Testing Library` will be used to test individual components.
    ```typescript
    // Example: src/components/Button.test.tsx
    import { render, screen } from '@testing-library/react';
    import { Button } from './Button';

    test('it should render the button with text', () => {
      render(<Button>Click Me</Button>);
      expect(screen.getByText('Click Me')).toBeInTheDocument();
    });
    ```
-   **E2E Tests**: Once major features are complete, Tauri's built-in `webdriver` support will be used for end-to-end tests that simulate a user clicking through the entire application flow.

## 7. CI/CD Pipeline (GitHub Actions)

The `.github/workflows/main.yml` file will define the following jobs:
1.  **`lint`**: Runs `clippy`, `rustfmt`, `eslint`, and `prettier`. Fails the build on any warning.
2.  **`test`**: Runs `cargo test` and `pnpm test`. Collects test coverage reports and uploads them as artifacts.
3.  **`build`**: On a separate matrix for `windows-latest`, `macos-latest`, and `ubuntu-latest`, builds the application.
4.  **`release`**: This job is triggered only on a git tag (e.g., `v1.0.0`). It runs the `build` job and then uses Tauri's `gh-release` action to create a GitHub Release with the compiled binaries attached.

This document represents the final, exhaustive plan. It integrates all specific details discussed and provides a clear, actionable blueprint for every major aspect of the project.

## 11. Project Management & Phasing

### 11.1. Project Scope
*   **In Scope**:
    *   All features defined in the User Stories (US-001, US-002, US-003).
    *   A fully installable desktop application for Windows, macOS, and Linux.
    *   Secure user authentication via Discord's official OAuth2 protocol.
    *   A clean, modern, and intuitive graphical user interface.
    *   Robust error handling, user feedback, and application logging.
*   **Out of Scope (for v1.0)**:
    *   A separate Command-Line Interface (CLI).
    *   Localization/Internationalization (UI will be in English).
    *   The "Keyword Filtering" feature mentioned as a potential v2 addition.
    *   User-configurable themes or a plugin/extension system.

### 11.2. Minimum Viable Product (MVP) Definition
The MVP is the smallest version of the product that can be shipped to provide immediate value to early adopters. The MVP for this project is defined as:

> A distributable and installable desktop application that allows a user to securely log in via Discord OAuth2 and use the **Bulk Message Deletion** feature, end-to-end, with a functional, clear user interface.

This focuses our efforts on delivering the most complex and high-value feature first, proving the viability of the entire architecture.
