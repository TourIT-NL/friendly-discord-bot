# Discord Purge OAuth2 Flow: Secure User Authentication for Discord Cleanup

This document details the robust and secure **OAuth2 authentication flow** implemented for user login within the **Discord Purge utility**. Ensuring secure access to your Discord account without ever handling your credentials directly is a paramount design principle for this **Discord message deletion and privacy management tool**. The flow leverages **PKCE (Proof Key for Code Exchange)** for enhanced security against authorization code interception attacks.

```
[React UI]          -> User clicks "Login with Discord" button in the Discord Purge desktop application.
[React UI]          -> The React frontend invokes a backend command: `invoke('start_oauth_flow')`.
[Rust Backend]      -> The `start_oauth_flow()` function in the Rust backend is called.
[Rust Backend]      -> Generates a cryptographically secure PKCE challenge and a unique state parameter for CSRF protection.
[Rust Backend]      -> Starts a temporary local HTTP server on a random free port (e.g., `http://localhost:58123`) to listen for the Discord redirect.
[Rust Backend]      -> Constructs the official Discord OAuth2 authorization URL, including `client_id`, required `scopes`, `PKCE challenge`, and the `state` parameter.
[Rust Backend]      -> Uses `tauri::api::shell::open(URL)` to open this constructed URL in the user's default web browser, initiating the secure authentication process.
----------------------------------------------------------------------------------
[User's Browser]    -> The user is redirected to Discord's official website. They log in (if not already) and see the Discord consent screen, detailing the permissions requested by the Discord Purge application. User clicks "Authorize".
[User's Browser]    -> Discord securely redirects the user's browser back to the temporary local server started by the Rust backend, appending the `authorization code` and `state` parameter to the callback URL (e.g., `http://localhost:58123/callback?code=...&state=...`).
----------------------------------------------------------------------------------
[Rust Backend]      -> The local server receives the callback request. It immediately validates the `state` parameter to prevent CSRF attacks.
[Rust Backend]      -> The temporary local server is shut down immediately after receiving and validating the callback.
[Rust Backend]      -> The Rust backend exchanges the received `authorization code` (along with the original `PKCE verifier`) for `access_token` and `refresh_token` via a secure POST request to the Discord API's token endpoint. This is a critical step in gaining secure **Discord API access**.
[Rust Backend]      -> Stores the `access_token` and `refresh_token` securely using OS-level storage (e.g., macOS Keychain, Windows Credential Manager, Linux Secret Service). These tokens are never stored in plaintext on disk.
[Rust Backend]      -> Emits a `window.emit('auth_success', user_profile)` event to the frontend, signaling successful authentication and optionally passing basic user profile information.
[React UI]          -> The React frontend listens for the 'auth_success' event, updates its Zustand store with the authentication status, and redirects the user to the main authenticated interface of the **Discord Purge application**.
```

This meticulous **OAuth2 implementation with PKCE** ensures that your **Discord login** is secure, and your tokens are handled with the highest level of care, making the **Discord Purge tool** a trustworthy solution for **Discord privacy management**.
