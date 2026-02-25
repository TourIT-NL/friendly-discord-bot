# 🔐 Architecture: Secure OAuth2 PKCE Flow

Security is our first law. This document details the cryptographic handshake used to connect **Discord Purge** to the Discord API without ever exposing user passwords.

---

## 🏗️ Technical Workflow

We implement **RFC 7636 (PKCE)**, which is the industry standard for secure desktop applications.

1.  **Initiation**: User clicks "Login". Frontend calls `invoke('start_oauth_flow')`.
2.  **Challenge Generation**: Rust generates a high-entropy `code_verifier` and a `code_challenge`.
3.  **Local Callback Server**: A temporary HTTP server starts on a random available port (e.g., `58123`).
4.  **Handoff**: `tauri::api::shell::open` sends the user to Discord's authorize URL with the `code_challenge` and a unique `state` string.
5.  **User Authorization**: User clicks "Authorize" on Discord's website.
6.  **Secure Redirect**: Discord redirects to `http://localhost:58123/callback?code=...&state=...`.
7.  **Validation**: Rust backend validates the `state` to prevent CSRF attacks.
8.  **Exchange**: Rust backend sends the `code` and the `code_verifier` to Discord's token endpoint via a secure HTTPS POST.
9.  **Storage**: Upon receiving the tokens, they are immediately stored in the **OS Keychain** (Encrypted).
10. **Success**: The local server shuts down, and `window.emit('auth_success')` is sent to the React frontend.

---

## 🛡️ Security Benefits

- **No Password Access**: The application never sees your login credentials.
- **Token Protection**: Tokens are never stored in plain text or local storage.
- **Interception Prevention**: PKCE ensures that even if the authorization code is stolen, it is useless without the secret verifier stored only in our memory.

---

## 🔑 Permissions Requested (Scopes)

- `identify`: To display your username and avatar.
- `guilds`: To list the servers you are in.
- `messages.read`: To scan for cleanup targets.
- `gdm.join`: To manage group conversations.

_Last updated: February 25, 2026_
